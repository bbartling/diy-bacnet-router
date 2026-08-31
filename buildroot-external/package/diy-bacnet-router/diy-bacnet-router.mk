################################################################################
#
# diy-bacnet-router
#
################################################################################

DIY_BACNET_ROUTER_VERSION = 0.1.0
DIY_BACNET_ROUTER_SITE = $(BR2_EXTERNAL_DIY_BACNET_ROUTER_PATH)/..
DIY_BACNET_ROUTER_SITE_METHOD = local
DIY_BACNET_ROUTER_LICENSE = MIT
DIY_BACNET_ROUTER_LICENSE_FILES = LICENSE
DIY_BACNET_ROUTER_CARGO_BUILD_OPTS = --package routerd --bin diy-bacnet-router --locked

define DIY_BACNET_ROUTER_INSTALL_TARGET_CMDS
	$(INSTALL) -D -m 0755 \
		$(@D)/target/$(RUSTC_TARGET_NAME)/release/diy-bacnet-router \
		$(TARGET_DIR)/usr/bin/diy-bacnet-router
	$(INSTALL) -D -m 0644 $(@D)/config/router.example.toml \
		$(TARGET_DIR)/etc/diy-bacnet-router/router.toml
	sed -i 's|web_root = "frontend/web/dist"|web_root = "/usr/share/diy-bacnet-router/web"|' \
		$(TARGET_DIR)/etc/diy-bacnet-router/router.toml
	$(INSTALL) -d -m 0755 $(TARGET_DIR)/usr/share/diy-bacnet-router/web
	if test -d $(@D)/frontend/web/dist; then \
		cp -a $(@D)/frontend/web/dist/. $(TARGET_DIR)/usr/share/diy-bacnet-router/web/; \
	else \
		$(INSTALL) -m 0644 $(@D)/frontend/fallback/index.html \
			$(TARGET_DIR)/usr/share/diy-bacnet-router/web/index.html; \
	fi
endef

define DIY_BACNET_ROUTER_INSTALL_INIT_SYSV
	$(INSTALL) -D -m 0755 \
		$(BR2_EXTERNAL_DIY_BACNET_ROUTER_PATH)/package/diy-bacnet-router/S80diy-bacnet-router \
		$(TARGET_DIR)/etc/init.d/S80diy-bacnet-router
endef

define DIY_BACNET_ROUTER_USERS
	dbr -1 dbr -1 * /var/lib/diy-bacnet-router /bin/false dialout DIY_BACnet_Router
endef

$(eval $(cargo-package))
