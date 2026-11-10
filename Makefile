.PHONY: dev release release-stop release-logs test seed lint reset-db
dev:
	docker compose up --build
release:
	docker compose -f docker-compose.release.yml up --build -d
release-stop:
	docker compose -f docker-compose.release.yml down
release-logs:
	docker compose -f docker-compose.release.yml logs -f api web
test:
	docker compose -f docker-compose.test.yml run --build --rm api-test
	docker compose -f docker-compose.test.yml run --rm web-test
seed:
	@echo "Development seed and migrations run automatically on API boot."
lint:
	cd apps/api && cargo fmt --check
	cd apps/web && npm run lint && npm run format:check && npm run typecheck
reset-db:
	@echo "Destructive: remove the development volume manually only after backing up required records. See docs/DEPLOYMENT.md."
