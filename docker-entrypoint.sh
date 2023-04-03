#!/usr/bin/env bash
set -Eeo pipefail
# TODO swap to -Eeuo pipefail above (after handling all potentially-unset variables)

# usage: file_env VAR [DEFAULT]
#    ie: file_env 'XYZ_DB_PASSWORD' 'example'
# (will allow for "$XYZ_DB_PASSWORD_FILE" to fill in the value of
#  "$XYZ_DB_PASSWORD" from a file, especially for Docker's secrets feature)
file_env() {
	local var="$1"
	local fileVar="${var}_FILE"
	local def="${2:-}"
	if [ "${!var:-}" ] && [ "${!fileVar:-}" ]; then
		echo >&2 "error: both $var and $fileVar are set (but are exclusive)"
		exit 1
	fi
	local val="$def"
	if [ "${!var:-}" ]; then
		val="${!var}"
	elif [ "${!fileVar:-}" ]; then
		val="$(< "${!fileVar}")"
	fi
	export "$var"="$val"
	unset "$fileVar"
}

# check to see if this file is being run or sourced from another script
_is_sourced() {
	# https://unix.stackexchange.com/a/215279
	[ "${#FUNCNAME[@]}" -ge 2 ] \
		&& [ "${FUNCNAME[0]}" = '_is_sourced' ] \
		&& [ "${FUNCNAME[1]}" = 'source' ]
}


# print large warning if PGP_DATA is emppty
docker_verify_minimum_env() {
	if [ -z "$PGP_DATA" ] ; then
		# The - option suppresses leading tabs but *not* spaces. :)
		cat >&2 <<-'EOE'
			Error: uninitialized license data is not specified.
			       You must specify PGP_DATA to a non-empty value for the
			       superuser. For example, "-e PGP_DATA=........" on "docker run".

		EOE
		exit 1
	fi
}

#
PGP_DATA=${PGP_DATA:-""}


# Loads various settings that are used elsewhere in the script
# This should be called before any other functions
docker_setup_env() { 
	file_env 'PGP_DATA'
	#file_env 'PGP_DATA' 'etc/ontp_license.data'

}

# check arguments for an option that would cause postgres to stop
# return true if there is one
_pg_want_help() {
	local arg
	for arg; do
		case "$arg" in
			-'?'|--help|--describe-config|-V|--version)
				return 0
				;;
		esac
	done
	return 1
}

_main() {
	# if first arg looks like a flag, assume we want to run postgres server
	docker_setup_env
	#
	if [ "${1:0:1}" = '-' ]; then
		set -- ontplserv "$@"
	fi

	if [ "$1" = 'ontplserv' ] && ! _pg_want_help "$@"; then
		# 
		if [ "$(id -u)" = '0' ]; then
			# then restart script as ontplserv user
			exec su-exec ontplserv "$BASH_SOURCE" "$@"
		fi

		docker_verify_minimum_env
	fi

	exec "$@"
}

if ! _is_sourced; then
	_main "$@"
fi
