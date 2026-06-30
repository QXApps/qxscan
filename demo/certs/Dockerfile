FROM alpine:3.20
RUN apk add --no-cache openssl bash
COPY generate.sh /generate.sh
RUN chmod +x /generate.sh
CMD ["/usr/bin/env", "bash", "/generate.sh"]
