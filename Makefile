.PHONY: help down up log

all: help

help:
	@echo "down - docker compose down"
	@echo "up - docker compose up --build -d"
	@echo "log - docker compose logs webhook"

down:
	docker compose down

up:
	docker compose up --build -d

log:
	docker compose logs webhook
