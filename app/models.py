"""Pydantic request/response models."""

from __future__ import annotations

from typing import Any, Literal, Optional

from pydantic import BaseModel, Field, field_validator, model_validator

DecodeLiteral = Literal["raw", "uint16", "int16", "uint32", "int32", "float32"]
FunctionLiteral = Literal["holding", "input"]


class BacnetReadRequest(BaseModel):
    device_instance: int = Field(..., ge=0)
    object_type: str = Field(..., examples=["analog-input"])
    object_instance: int = Field(..., ge=0)
    property_id: str = Field(default="present-value")


class BacnetRpmPropertySpec(BaseModel):
    property_id: str
    array_index: Optional[int] = None


class BacnetRpmObjectSpec(BaseModel):
    object_type: str
    object_instance: int
    properties: list[BacnetRpmPropertySpec]


class BacnetRpmRequest(BaseModel):
    device_instance: int
    objects: list[BacnetRpmObjectSpec] = Field(..., min_length=1, max_length=32)


class BacnetWhoisRequest(BaseModel):
    low: Optional[int] = Field(default=0, ge=0)
    high: Optional[int] = Field(default=4_194_303, ge=0)


ValueTypeLiteral = Literal[
    "real", "double", "unsigned", "signed", "enumerated", "boolean", "character_string", "null"
]


class BacnetWriteRequest(BaseModel):
    """WriteProperty with optional priority; send value null (or "null") to relinquish."""

    device_instance: int = Field(..., ge=0)
    object_type: str = Field(..., examples=["analog-output"])
    object_instance: int = Field(..., ge=0)
    property_id: str = Field(default="present-value")
    value: Optional[Any] = Field(
        default=None,
        description='Value to write; JSON null or "null" releases the override at the given priority.',
    )
    priority: Optional[int] = Field(
        default=None, ge=1, le=16, description="Priority slot 1-16 (required to release with null)."
    )
    value_type: Optional[ValueTypeLiteral] = Field(
        default=None,
        description="Force encoding type. Default: float->real, bool->enumerated, str->character_string.",
    )

    @model_validator(mode="after")
    def _release_needs_priority(self) -> "BacnetWriteRequest":
        is_release = self.value is None or (
            isinstance(self.value, str) and self.value.strip().lower() == "null"
        )
        if is_release and self.priority is None:
            raise ValueError("Releasing (null) requires a priority (1-16)")
        return self


class BacnetObjectRef(BaseModel):
    device_instance: int = Field(..., ge=0)
    object_type: str = Field(..., examples=["analog-output"])
    object_instance: int = Field(..., ge=0)


class DeviceInstanceRequest(BaseModel):
    device_instance: int = Field(..., ge=0, examples=[5007])


class ServerUpdatePointsRequest(BaseModel):
    """Name -> new present-value for hosted (writable) server points."""

    updates: dict[str, Any] = Field(..., examples=[{"openfdd-active-fault-count": 3}])


class ServerScheduleUpdateRequest(BaseModel):
    object_instance: int = Field(..., ge=0)
    schedule_default: Optional[Any] = None
    weekly_schedule: Optional[list[list[dict[str, Any]]]] = Field(
        default=None,
        description="7 lists (Mon-Sun) of {time: 'HH:MM:SS', value: number} entries.",
    )


class ModbusRegisterOp(BaseModel):
    address: int = Field(..., ge=0, le=65535)
    count: int = Field(default=1, ge=1, le=125)
    function: FunctionLiteral = "holding"
    decode: Optional[DecodeLiteral] = None
    scale: Optional[float] = None
    offset: Optional[float] = None
    label: Optional[str] = None

    @model_validator(mode="after")
    def decode_needs_word_count(self) -> ModbusRegisterOp:
        if self.decode in ("float32", "uint32", "int32") and self.count < 2:
            raise ValueError(f"decode={self.decode!r} requires count >= 2")
        return self


class ModbusReadRequest(BaseModel):
    host: str
    port: int = Field(default=502, ge=1, le=65535)
    unit_id: int = Field(default=1, ge=0, le=255)
    timeout: float = Field(default=5.0, ge=0.5, le=60.0)
    registers: list[ModbusRegisterOp] = Field(..., min_length=1, max_length=32)

    @field_validator("host")
    @classmethod
    def strip_host(cls, v: str) -> str:
        v = (v or "").strip()
        if not v:
            raise ValueError("host must be non-empty")
        return v


class HaystackReadRequest(BaseModel):
    filter: str = Field(default="site", examples=["site", "point and temp"])


class HaystackNavRequest(BaseModel):
    nav_id: Optional[str] = Field(default=None, description="Haystack nav id or null for root")


class HaystackHisReadRequest(BaseModel):
    ids: list[str] = Field(..., min_length=1, max_length=64)
    range_start: Optional[str] = None
    range_end: Optional[str] = None


class WeatherResponse(BaseModel):
    temp_f: float
    humidity: float
    wind_mph: float
    dewpoint_f: float
    location: str
    from_api: bool
    reason: str = ""
    updated_at: Optional[str] = None
