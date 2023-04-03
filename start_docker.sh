#!/bin/sh -x
#
#


#
sudo docker run -it --rm --network host --name validate_pgp_server \
        -v /etc/localtime:/etc/localtime:ro \
        -v "$PWD/certs/server.crt:/var/validate_pgp_server/tls/client.crt:ro" \
        -v "$PWD/certs/server.key:/var/validate_pgp_server/tls/client.key:ro" \
        -v "$PWD/etc/validate_pgp_server.json:/var/validate_pgp_server/etc/validate_pgp_server.json:ro" \
        validate_pgp_server:2.0.4
