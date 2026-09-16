# Contributing to zk-pool

Спасибо за интерес к проекту! Этот документ описывает, как контрибьютить в zk-pool.

## Содержание

- [Кодекс поведения](#кодекс-поведения)
- [Как я могу помочь](#как-я-могу-помочь)
- [Настройка окружения](#настройка-окружения)
- [Git flow](#git-flow)
- [Стиль кода](#стиль-кода)
- [Коммиты](#коммиты)
- [Pull Requests](#pull-requests)
- [Тесты](#тесты)

---

## Кодекс поведения

Будь вежлив. Уважай других. Конструктивная критика приветствуется, личные оскорбления — нет.

---

## Как я могу помочь

- 🐛 **Найти и починить баг** — [открой issue](https://github.com/kwebhub/private-transfer/issues/new?template=bug_report.md)
- ✨ **Предложить фичу** — [открой feature request](https://github.com/kwebhub/private-transfer/issues/new?template=feature_request.md)
- 📖 **Улучшить документацию** — PR с правками в `docs/` или README
- 🧪 **Написать тесты** — см. [Этап 4 в roadmap](PROJECT_CONTEXT.md#14-roadmap)
- 🔒 **Найти уязвимость** — см. [SECURITY.md](SECURITY.md)

---

## Настройка окружения

Смотри [docs/deployment.md](docs/deployment.md) — там **полная** инструкция.

**Кратко:**

```bash
git clone https://github.com/kwebhub/private-transfer.git
cd private-transfer

# 1. Docker-окружение
docker compose up -d

# 2. Клонировать Sunspot (внешний репо)
cd workspace && git clone https://github.com/reilabs/sunspot.git && cd ..

# 3. Собрать программу
cd workspace/ptrans && anchor build && cd ../..

# 4. Применить миграции
docker compose exec -T postgres psql -U ptrans -d ptrans < workspace/ptrans/app/backend/migrations/001_init.sql

# 5. Установить зависимости
cd workspace/ptrans/app/frontend && pnpm install && cd ../../..
cd workspace/ptrans/app/merkle && pnpm install && cd ../../..

# 6. Запустить всё
./start-all.sh
```

---

## Git flow

Мы используем **trunk-based development** с короткими feature-ветками.

### Ветки

| Ветка | Назначение |
|-------|-----------|
| `main` | **Production-ready**. Защищена. Только через PR. |
| `feat/<name>` | Новая фича. Ветвится от `main`. |
| `fix/<name>` | Исправление бага. Ветвится от `main`. |
| `docs/<name>` | Документация. Ветвится от `main`. |
| `chore/<name>` | Обновления, конфиги, CI. Ветвится от `main`. |
| `refactor/<name>` | Рефакторинг без изменения поведения. |

### Рабочий процесс

```bash
# 1. Обновить main
git checkout main
git pull origin main

# 2. Создать feature-ветку
git checkout -b feat/my-feature

# 3. Работать
git add .
git commit -m "feat: add my feature"

# 4. Обновить относительно main (rebase, не merge)
git fetch origin
git rebase origin/main

# 5. Запушить
git push origin feat/my-feature

# 6. Открыть PR на GitHub
```

### Ребейз, не мерж

**Всегда** используй `git rebase origin/main`, а не `git merge main`. Это сохраняет **линейную** историю.

---

## Стиль кода

### Rust

- **`rustfmt`** — обязателен. `cargo fmt`.
- **`clippy`** — обязателен. `cargo clippy --all-targets -- -D warnings`.
- **`edition 2021`**.
- **Doc-comments** для публичных функций (`///`).
- **Ошибки** через `thiserror` (не `Box<dyn Error>` в продакшене).
- **Логи** через `tracing` (не `println!`).

### TypeScript / Vue

- **`eslint`** + **`oxlint`** — обязательны.
- **`prettier`** — форматирование.
- **`vue-tsc`** — type-checking.
- **Composition API** (`<script setup>`).
- **Pug** для шаблонов.
- **SCSS** для стилей.

### Noir

- **`nargo fmt`** — форматирование.
- **`nargo test`** — тесты.
- Комментарии на английском.

### YAML / Markdown

- **2 пробела** для отступов.
- **LF** line endings.
- **UTF-8**.
- См. `.editorconfig`.

---

## Коммиты

Мы используем [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Типы

| Тип | Когда |
|-----|-------|
| `feat` | Новая функциональность |
| `fix` | Исправление бага |
| `docs` | Только документация |
| `style` | Форматирование (без изменения логики) |
| `refactor` | Рефакторинг (без изменения поведения) |
| `perf` | Оптимизация производительности |
| `test` | Добавление/изменение тестов |
| `build` | Сборка, зависимости |
| `ci` | CI/CD |
| `chore` | Прочее (обновления, конфиги) |

### Scope (опционально)

`backend`, `prover`, `merkle`, `frontend`, `program`, `circuits`, `docs`, `ci`, `deps`.

### Примеры

```
feat(backend): add /api/proof endpoint
fix(frontend): handle Solflare network mismatch
docs: add threat model
ci: pin Anchor to 0.32.2
chore(deps): bump axum from 0.7.5 to 0.7.6
refactor(backend): extract config to Config struct
```

---

## Pull Requests

1. **Заполни шаблон** (`.github/PULL_REQUEST_TEMPLATE.md`).
2. **Свяжи с issue**: `Closes #123`.
3. **Один PR = одна задача.** Не смешивай несколько фич.
4. **CI должен быть зелёным** до ревью.
5. **Обнови ветку** через `git rebase origin/main` (не merge).
6. **Отвечай на комментарии** ревьюера.
7. **Squash & Merge** — если несколько коммитов, они схлопнутся в один.

### Ревью

- Минимум **1 approve** для merge.
- **main** защищён: нельзя пушить напрямую.
- CI должен пройти **все 6 jobs**.

---

## Тесты

### Backend

```bash
cd workspace/ptrans/app/backend
cargo test
```

### Программа

```bash
cd workspace/ptrans
anchor test
```

### Noir

```bash
cd workspace/ptrans/app/circuits/withdrawal
nargo test
```

### Frontend

```bash
cd workspace/ptrans/app/frontend
pnpm test:unit    # vitest
pnpm test:e2e     # playwright
```

**Требования:** новые фичи **должны** иметь тесты. Баг-фиксы **должны** иметь regression-тест.

---

## Вопросы

- **Общие вопросы:** [Discussions](https://github.com/kwebhub/private-transfer/discussions)
- **Баги:** [Issues](https://github.com/kwebhub/private-transfer/issues)
- **Безопасность:** [SECURITY.md](SECURITY.md)

Спасибо за вклад! 🎉
