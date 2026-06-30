#!/usr/bin/env python3
"""
SMTP TLS stub server for QXScan demo.
Accepts SMTP connections on port 587 with STARTTLS, logs emails, discards them.

Uses aiosmtpd with a custom factory that attaches tls_context for STARTTLS support.
"""
import asyncio
import ssl
import logging
from aiosmtpd.controller import Controller

logging.basicConfig(level=logging.INFO, format='[mail] %(message)s')
logger = logging.getLogger('mail')


class QXScanMailHandler:
    """Handler that logs and discards all incoming email."""

    async def handle_DATA(self, server, session, envelope):
        logger.info(f"=== Mail received ===")
        logger.info(f"From: {envelope.mail_from}")
        logger.info(f"To: {', '.join(envelope.rcpt_tos)}")
        content = envelope.content.decode('utf-8', errors='replace')
        logger.info(f"Size: {len(content)} bytes")
        # Log first 2KB of content
        logger.info(f"Content preview:\n{content[:2000]}")
        logger.info(f"=== End of mail ===")
        return '250 Message accepted for delivery'


class STARTTLSController(Controller):
    """Controller that attaches TLS context for STARTTLS (not implicit TLS)."""

    def factory(self):
        # Create an SMTP instance with TLS context for STARTTLS
        smtp = super().factory()
        smtp.tls_context = self.ssl_context
        return smtp


def main():
    cert_path = '/certs/mail.crt'
    key_path = '/certs/mail.key'

    # Create SSL context for STARTTLS
    ssl_context = ssl.create_default_context(ssl.Purpose.CLIENT_AUTH)
    ssl_context.load_cert_chain(cert_path, key_path)
    ssl_context.minimum_version = ssl.TLSVersion.TLSv1_2

    handler = QXScanMailHandler()
    controller = STARTTLSController(
        handler,
        hostname='0.0.0.0',
        port=587,
        ssl_context=ssl_context,  # Used for STARTTLS via tls_context in factory()
        server_hostname='mail',
    )

    logger.info("=" * 50)
    logger.info("QXScan Mail Stub Server")
    logger.info(f"Listening on 0.0.0.0:587 (SMTP with STARTTLS)")
    logger.info(f"Certificate: mail.crt")
    logger.info("=" * 50)

    controller.start()

    try:
        asyncio.get_event_loop().run_forever()
    except KeyboardInterrupt:
        logger.info("Shutting down...")
        controller.stop()


if __name__ == '__main__':
    main()
