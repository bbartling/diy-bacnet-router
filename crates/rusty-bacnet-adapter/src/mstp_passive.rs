//! Receive-only MS/TP acceptance gate (no master TX).
//!
//! Opens the appliance serial path and decodes frames with public
//! `bacnet_transport::mstp_frame::decode_frame_stream`. Transmits zero bytes.
//! Fails closed on silence, missing expected source, or missing token/PFM activity.

use std::collections::BTreeSet;
use std::path::Path;
use std::time::{Duration, Instant};

use bacnet_transport::mstp::SerialPort;
use bacnet_transport::mstp_frame::{decode_frame_stream, FrameType, StreamDecode, PREAMBLE};
use tokio::time::timeout;
use tracing::info;

use crate::mstp_qualify::open_appliance_serial;
use crate::ports::MstpTransportParams;
use crate::validate::AdapterError;
use crate::UPSTREAM_REVISION;

/// Thresholds for a fail-closed passive observation window.
#[derive(Debug, Clone)]
pub struct MstpPassiveCriteria {
    pub expect_source: u8,
    pub min_complete_frames: u64,
    pub min_valid_ratio: f64,
}

impl Default for MstpPassiveCriteria {
    fn default() -> Self {
        Self {
            expect_source: 2,
            min_complete_frames: 10,
            min_valid_ratio: 0.99,
        }
    }
}

/// Observed counters from an RX-only window.
#[derive(Debug, Default, Clone)]
pub struct MstpPassiveReport {
    pub ok: bool,
    pub serial: String,
    pub baud: u32,
    pub seconds: u64,
    pub expect_source: u8,
    pub rx_bytes: u64,
    pub complete_frames: u64,
    pub tokens: u64,
    pub poll_for_master: u64,
    pub data_frames: u64,
    pub invalid: u64,
    pub need_more_stalls: u64,
    pub valid_ratio: f64,
    pub sources_seen: Vec<u8>,
    pub failure_reason: String,
    pub error: Option<String>,
}

impl MstpPassiveReport {
    /// Apply fail-closed scoring. Does not invent traffic.
    pub fn score(&mut self, criteria: &MstpPassiveCriteria) {
        let denom = self.complete_frames + self.invalid;
        self.valid_ratio = if denom == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                self.complete_frames as f64 / denom as f64
            }
        };

        let has_source = self.sources_seen.contains(&criteria.expect_source);
        let has_token_or_pfm = self.tokens > 0 || self.poll_for_master > 0;
        let mut failures = Vec::new();
        if self.rx_bytes == 0 {
            failures.push("rx_bytes=0 (silence)");
        }
        if self.complete_frames < criteria.min_complete_frames {
            failures.push("complete_frames below minimum");
        }
        if !has_source {
            failures.push("expected source not seen");
        }
        if !has_token_or_pfm {
            failures.push("no token or Poll-For-Master activity");
        }
        if self.valid_ratio < criteria.min_valid_ratio {
            failures.push("valid_ratio below minimum");
        }
        if self.error.is_some() {
            failures.push("serial read error");
        }

        self.expect_source = criteria.expect_source;
        if failures.is_empty() {
            self.ok = true;
            self.failure_reason.clear();
        } else {
            self.ok = false;
            self.failure_reason = failures.join("; ");
        }
    }

    /// Atomic JSON report (temp + rename).
    pub fn write_json(&self, path: &Path) -> Result<(), AdapterError> {
        let sources = self
            .sources_seen
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        let err = self
            .error
            .as_ref()
            .map(|e| format!("{e:?}"))
            .unwrap_or_else(|| "null".to_owned());
        let body = format!(
            "{{\n\
               \"ok\": {},\n\
               \"ready_to_route\": false,\n\
               \"mode\": \"mstp_passive\",\n\
               \"tx_bytes\": 0,\n\
               \"serial\": {:?},\n\
               \"baud\": {},\n\
               \"seconds\": {},\n\
               \"expect_source\": {},\n\
               \"upstream_sha\": {UPSTREAM_REVISION:?},\n\
               \"rx_bytes\": {},\n\
               \"complete_frames\": {},\n\
               \"tokens\": {},\n\
               \"poll_for_master\": {},\n\
               \"data_frames\": {},\n\
               \"invalid\": {},\n\
               \"need_more_stalls\": {},\n\
               \"valid_ratio\": {},\n\
               \"sources_seen\": [{sources}],\n\
               \"failure_reason\": {:?},\n\
               \"error\": {err}\n\
             }}\n",
            self.ok,
            self.serial,
            self.baud,
            self.seconds,
            self.expect_source,
            self.rx_bytes,
            self.complete_frames,
            self.tokens,
            self.poll_for_master,
            self.data_frames,
            self.invalid,
            self.need_more_stalls,
            self.valid_ratio,
            self.failure_reason,
        );
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, body)
            .map_err(|e| AdapterError::Validation(format!("passive report write: {e}")))?;
        std::fs::rename(&tmp, path)
            .map_err(|e| AdapterError::Validation(format!("passive report rename: {e}")))?;
        Ok(())
    }
}

/// Run an RX-only observation window against the configured appliance serial.
pub async fn run_mstp_passive(
    params: &MstpTransportParams,
    seconds: u64,
    criteria: MstpPassiveCriteria,
) -> Result<MstpPassiveReport, AdapterError> {
    let serial = open_appliance_serial(params)?;
    let mut report = MstpPassiveReport {
        serial: params.serial_path.clone(),
        baud: params.baud_rate,
        seconds,
        expect_source: criteria.expect_source,
        ..MstpPassiveReport::default()
    };

    info!(
        path = %params.serial_path,
        baud = params.baud_rate,
        seconds,
        expect_source = criteria.expect_source,
        "MS/TP passive RX-only session started (zero TX)"
    );

    let deadline = Instant::now() + Duration::from_secs(seconds);
    let mut frame_buf: Vec<u8> = Vec::with_capacity(2048);
    let mut recv = vec![0u8; 2048];
    let mut sources = BTreeSet::new();

    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match timeout(remaining, serial.read(&mut recv)).await {
            Err(_) => break,
            Ok(Ok(0)) => {}
            Ok(Ok(n)) => {
                report.rx_bytes += n as u64;
                frame_buf.extend_from_slice(&recv[..n]);
                loop {
                    match decode_frame_stream(&frame_buf) {
                        StreamDecode::NeedMore => {
                            report.need_more_stalls += 1;
                            break;
                        }
                        StreamDecode::Complete { frame, consumed } => {
                            report.complete_frames += 1;
                            sources.insert(frame.source);
                            match frame.frame_type {
                                FrameType::Token => report.tokens += 1,
                                FrameType::PollForMaster => report.poll_for_master += 1,
                                FrameType::BACnetDataExpectingReply
                                | FrameType::BACnetDataNotExpectingReply => {
                                    report.data_frames += 1;
                                }
                                _ => {}
                            }
                            frame_buf.drain(..consumed);
                        }
                        StreamDecode::Invalid { discard } => {
                            report.invalid += 1;
                            let d = discard.max(1).min(frame_buf.len());
                            frame_buf.drain(..d);
                        }
                    }
                }
                if frame_buf.len() == 1 && frame_buf[0] == PREAMBLE[0] {
                    // keep lone 0x55
                } else if frame_buf.len() > 4096 {
                    frame_buf.clear();
                }
            }
            Ok(Err(e)) => {
                report.error = Some(e.to_string());
                break;
            }
        }
    }

    // Drop serial without ever calling write — TokioSerialPort closes on drop.
    drop(serial);

    report.sources_seen = sources.into_iter().collect();
    report.score(&criteria);
    info!(
        ok = report.ok,
        rx_bytes = report.rx_bytes,
        complete = report.complete_frames,
        tokens = report.tokens,
        pfm = report.poll_for_master,
        sources = ?report.sources_seen,
        failure = %report.failure_reason,
        "MS/TP passive RX-only session finished"
    );
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_fail_closed_on_silence() {
        let mut report = MstpPassiveReport::default();
        report.score(&MstpPassiveCriteria::default());
        assert!(!report.ok);
        assert!(report.failure_reason.contains("silence"));
    }

    #[test]
    fn scores_pass_with_expected_source_and_pfm() {
        let mut report = MstpPassiveReport {
            rx_bytes: 100,
            complete_frames: 20,
            poll_for_master: 20,
            sources_seen: vec![2],
            ..MstpPassiveReport::default()
        };
        report.score(&MstpPassiveCriteria::default());
        assert!(report.ok, "{}", report.failure_reason);
    }

    #[test]
    fn scores_fail_without_expected_source() {
        let mut report = MstpPassiveReport {
            rx_bytes: 100,
            complete_frames: 20,
            tokens: 5,
            sources_seen: vec![7],
            ..MstpPassiveReport::default()
        };
        report.score(&MstpPassiveCriteria::default());
        assert!(!report.ok);
        assert!(report.failure_reason.contains("expected source"));
    }
}
