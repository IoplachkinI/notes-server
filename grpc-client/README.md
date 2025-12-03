# gRPC Клиент

Легковесный простой gRPC клиент для проверки всех методов сервера заметок

## Сборка и запуск (через cargo)

```bash
cargo build
```

<Запускаем note-server>

```bash
cargo run
```

По дефолту клиент стучится по этому адресу `http://127.0.0.1:5000` - здесь по дефолту слушает gRPC сервер из своего контейнера. Этот адрес можно поменять:

```bash
GRPC_SERVER_ADDR=http://example.com:50051 cargo run
```

## Запуск через docker compose

```bash
docker-compose up --build
```

По дефолту порт для сервера выставлен тот же, `http://127.0.0.1:5000`

Его также можно поменять:

```bash
GRPC_SERVER_ADDR=http://server-dev:50051 docker-compose up --build
```

