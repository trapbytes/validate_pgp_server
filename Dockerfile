##
##
ARG UBUNTU_VERSION=20.04
FROM ubuntu:${UBUNTU_VERSION}

# Our labels
LABEL validate_pgp_server.release="1.0.1" \
      validate_pgp_server.release-date="2023-02-23" \
      validate_pgp_server.release-type="production" \
      validate_pgp_server.description="Validate pgp encrypted data Server"
#
## set/get locales
RUN set -eux; \
	\
	export DEBIAN_FRONTEND=noninteractive; \
	export DEBCONF_NONINTERACTIVE_SEEN=true; \
	apt-get update && apt-get install -y locales && rm -rf /var/lib/apt/lists/* \
	&& localedef -i en_US -c -f UTF-8 -A /usr/share/locale/locale.alias en_US.UTF-8

#
# make the "en_US.UTF-8" locale so postgres will be utf-8 enabled by default
ENV LANG en_US.utf8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8
#

# We need full control over the running user, including the UID, therefore we
# create the ontplserv user
#
RUN set -eux; \
	\
	export DEBIAN_FRONTEND=noninteractive; \
	export DEBCONF_NONINTERACTIVE_SEEN=true; \
	\
	addgroup --system --gid 106 pgplserv; \
	adduser --uid 106 -gid 106 --home /home/pgplserv --shell /bin/sh pgplserv; \
	mkdir -p /opt/local/build/lserv ; \
	chown -R pgplserv:pgplserv /opt/local/build/ ; \
	\
	apt-get update && apt-get install -y \
	bash \
	curl \
	openssl \
	libssl-dev \
	autoconf \
	automake \
	gnupg \
	gcc \
	g++ \
	pkg-config \
	clang \
	llvm \
	pkg-config \
	nettle-dev

#
#
COPY docker-entrypoint.sh /usr/local/bin/
# backwards compat - linking of docker-entrypoint.sh to /
RUN ln -s /usr/local/bin/docker-entrypoint.sh /


COPY /src /opt/local/build/lserv/src/
COPY /Cargo.toml /opt/local/build/lserv/

# expose the default port
EXPOSE 2568/tcp

#need to execute and install rust as desired userid
USER ontplserv
#
#
RUN set -eux; \
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y ; \
	cd /opt/local/build/ ; \
	#
	~/.cargo/bin/cargo new validate_pgp_server ; \
	cp /opt/local/build/lserv/Cargo.toml validate_pgp_server ; \
	cp -R /opt/local/build/lserv/src/* validate_pgp_server/src/ ; \
	cd validate_pgp_server ; \
	~/.cargo/bin/cargo build --release

# need root to remove the pkgs and clean up
USER root
RUN set -eux; \
	apt-get remove -y --allow-unauthenticated \
		gcc \
		g++ \
		autoconf \
		automake \
		curl ; \
	apt-get autoremove -y ; \
	rm -rf /opt/local/build/lserv/* ; \
	rm -rf /opt/local/build/validate_pgp_server/src/* ; \
	rm -rf /var/lib/apt/lists/*

#
ENTRYPOINT ["docker-entrypoint.sh"]

#
# reset our user to be deired user id
USER pgplserv

#
## launch command
CMD ["/opt/local/build/validate_pgp_server/target/release/validate_pgp_server", \
     "--config_file", \
     "/var/validate_pgp_server/etc/validate_pgp_server.json"]
