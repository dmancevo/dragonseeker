FROM ghcr.io/astral-sh/uv:0.8.17-python3.13-alpine

WORKDIR app/

COPY ./app .
RUN uv sync --no-dev

EXPOSE 8000
CMD ["uv", "run", "fastapi", "run", "app.py"]