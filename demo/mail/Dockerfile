FROM python:3.12-alpine
RUN pip install --no-cache-dir aiosmtpd
COPY server.py /server.py
EXPOSE 587
CMD ["python", "/server.py"]
