FROM caddy:2.7.6-alpine

COPY Caddyfile /etc/caddy/Caddyfile
COPY ui/dist /srv
