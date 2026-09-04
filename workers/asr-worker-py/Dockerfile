FROM python:3.12-slim

WORKDIR /app

COPY requirements-lock.txt .
RUN pip install --no-cache-dir --no-deps -r requirements-lock.txt \
    && pip check

COPY main.py .
COPY input.schema.json output.schema.json error.schema.json ./

CMD ["python", "main.py"]
