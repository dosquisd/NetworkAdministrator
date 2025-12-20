#!/bin/bash

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

source $PROJECT_ROOT/scripts/.env

# Stop and delete existing container if it exists
docker stop pihole 2>/dev/null || true
docker rm pihole 2>/dev/null || true

docker run \
    --name pihole \
    -p 53:53/tcp \
    -p 53:53/udp \
    -p "80:80/tcp" \
    -p 443:443/tcp \
    -e TZ=America/Bogota \
    -e FTLCONF_webserver_api_password=$PIHOLE_PASSWORD \
    -e FTLCONF_dns_listeningMode=all \
    -v $PROJECT_ROOT/etc-pihole:/etc/pihole \
    -v $PROJECT_ROOT/etc-dnsmasq.d:/etc/dnsmasq.d \
    -d \
    pihole/pihole:latest
