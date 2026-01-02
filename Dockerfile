FROM python:3.11-slim

WORKDIR /app

RUN apt-get update && apt-get install -y libpq-dev gcc && rm -rf /var/lib/apt/lists/*

COPY pyproject.toml .
COPY README.md .

RUN pip install --no-cache-dir -e .

COPY src/ ./src/

EXPOSE 8888

CMD ["python", "-u", "src/server/app.py"]