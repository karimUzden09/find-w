# Руководство по интеграционным тестам

В проекте используются интеграционные тесты, которые проверяют полный путь запроса:

1. роутинг HTTP в `axum`
2. extractors (включая JWT-авторизацию)
3. обработчики (handlers) и формирование ответа
4. SQL-запросы к реальной Postgres

По вашему требованию исключён только модуль `notes`.

## Где лежат тесты

- `tests/common/mod.rs`: общий тестовый harness (`TestApp`)
- `tests/core_api.rs`: health, db-health, me, docs/openapi
- `tests/auth_api.rs`: register/login/refresh/logout
- `tests/groups_api.rs`: create/list/delete groups
- `tests/vk_tokens_api.rs`: add/delete VK tokens + проверки хранения

## Как работают тесты

### 1) Изолированная БД на каждый тест (`#[sqlx::test]`)

Каждый тест помечен `#[sqlx::test]` и получает `PgPool`:

```rust
#[sqlx::test]
async fn some_test(pool: PgPool) { ... }
```

`sqlx` создаёт временную отдельную БД, применяет миграции и передаёт пул в тест.
Это изолирует тесты друг от друга и не загрязняет основную БД приложения.

### 2) HTTP-запросы внутри процесса (`Router::oneshot`)

`TestApp` собирает реальный роутер через `build_router(state)` и отправляет запросы прямо в него:

```rust
let response = app.clone().oneshot(request).await?;
```

Порт не поднимается, но путь запроса полностью реальный:

1. router
2. middleware/extractors
3. handler
4. DB

## Требования для запуска

1. Запущенный Postgres
2. Переменная `DATABASE_URL` указывает на Postgres
3. У пользователя БД есть права на создание/удаление временных тестовых БД

Пример:

```bash
export DATABASE_URL=postgres://app:app@localhost:5432/app
```

## Запуск тестов

Все интеграционные тесты:

```bash
cargo test --tests
```

Один файл:

```bash
cargo test --test auth_api
```

Один тест:

```bash
cargo test --test auth_api refresh_rotates_tokens_and_old_refresh_becomes_invalid
```

## Тестовый harness (`tests/common/mod.rs`)

`TestApp` даёт готовые helper-методы:

1. `post_json(path, body, bearer)` -> `(StatusCode, Value)`
2. `get_json(path, bearer)` -> `(StatusCode, Value)`
3. `get_text(path, bearer)` -> `(StatusCode, String)`
4. `delete_json(path, body, bearer)` -> `(StatusCode, Value)`
5. `delete(path, bearer)` -> `StatusCode`
6. `register_and_login()` -> тестовый пользователь с `access_token` и `refresh_token`

Это убирает повторяющийся boilerplate в тестах.

## Что покрыто сейчас

### Core

1. `GET /health` возвращает `ok`
2. `GET /db-health` возвращает `ok`
3. `GET /me` требует авторизацию
4. эндпоинты документации отвечают (`/docs`, `/api-docs/openapi.json`)

### Auth

1. happy-path: register + login + `/me`
2. конфликт при повторной регистрации того же email
3. unauthorized при неверном пароле
4. refresh-ротация: старый refresh становится недействительным
5. logout ревокает refresh token

### Groups

1. create/list/delete группы
2. изоляция по владельцу (чужой пользователь не может удалить)
3. валидация payload и проверка авторизации

### VK Tokens

1. add/delete токенов
2. дедупликация (`inserted`/`skipped`)
3. проверка шифрованного хранения (`token_encrypted` не plaintext)
4. проверка внутреннего метода чтения/дешифровки
5. валидация payload и проверка авторизации

## Как добавить новый тест-кейс (мини-туториал)

Пример: добавить негативный кейс для `POST /groups` с невалидным payload.

### Шаг 1: выберите файл

Откройте `tests/groups_api.rs`.

### Шаг 2: создайте тест с изолированной БД

```rust
#[sqlx::test]
async fn groups_create_rejects_invalid_payload(pool: PgPool) { ... }
```

### Шаг 3: соберите app и пользователя

```rust
let app = TestApp::new(pool);
let user = app.register_and_login().await;
```

### Шаг 4: отправьте запрос

```rust
let (status, body) = app
    .post_json("/groups", json!({ "group_id": 0 }), Some(&user.access_token))
    .await;
```

### Шаг 5: проверьте результат

```rust
assert_eq!(status, StatusCode::BAD_REQUEST);
assert_eq!(body.get("error").and_then(Value::as_str), Some("BAD_REQUEST"));
```

### Шаг 6: сначала запустите файл с тестом

```bash
cargo test --test groups_api
```

### Шаг 7: затем прогоните весь integration-набор

```bash
cargo test --tests
```

## Рекомендации по стилю тестов

1. Один тест = одно проверяемое поведение
2. Говорящие имена (`what_happens_when_condition`)
3. Использовать helper-методы harness вместо дублирования запроса
4. Проверять и статус-код, и важные поля ответа
5. Для security-логики добавлять прямые проверки в БД

## Частые проблемы

1. `error communicating with database`
   Проверьте, что Postgres запущен и `DATABASE_URL` корректен.
2. Нет прав на создание тестовых БД
   Выдать пользователю БД право `CREATEDB`.
3. Несоответствие SQLx после изменения схемы
   Примените миграции перед тестами:

```bash
sqlx migrate run
```
