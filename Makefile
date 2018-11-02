PREFIX = /usr/local

.PHONY: all dmenu-quicklaunch dmenu-quicklaunch-srv install clean
all: dmenu-quicklaunch dmenu-quicklaunch-srv

dmenu-quicklaunch:
	cargo build --release --bin dmenu-quicklaunch

dmenu-quicklaunch-srv:
	cargo build --release --bin dmenu-quicklaunch-srv

install:
	mkdir -p $(DESTDIR)$(PREFIX)/bin
	install -m755 target/release/dmenu-quicklaunch $(DESTDIR)$(PREFIX)/bin/
	install -m755 target/release/dmenu-quicklaunch-srv $(DESTDIR)$(PREFIX)/bin/
	install -m644 dmenu-quicklaunch-srv.service $(DESTDIR)/usr/lib/systemd/user/

systemd-enable:
	systemctl --user enable dmenu-quicklaunch-srv.service
	systemctl --user start dmenu-quicklaunch-srv.service

clean:
	@rm -r target/

systemd-disable:
	systemctl --user disable dmenu-quicklaunch-srv.service

uninstall:
	rm $(DESTDIR)$(PREFIX)/bin/dmenu-quicklaunch
	rm $(DESTDIR)$(PREFIX)/bin/dmenu-quicklaunch-srv
	rm $(DESTDIR)/usr/lib/systemd/user/dmenu-quicklaunch-srv.service
