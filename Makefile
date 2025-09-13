docker-compose-up-support:
	docker compose -p "node-support" -f ./infra/docker/docker-compose.yml up -d --remove-orphans

docker-compose-up:
	docker compose -p "node-support" -f ./infra/docker/docker-compose.yml up -d

docker-compose-down:
	docker compose -p "node-support" -f ./infra/docker/docker-compose.yml down


setup-node-db-with-migration: docker-compose-up
	./infra/scripts/run-migrations.sh

start-node: docker-compose-up
	./infra/scripts/start-node.sh

start-node1:
	./infra/scripts/run-migrations1.sh
	./infra/scripts/start-node1.sh

start-node2:
	./infra/scripts/run-migrations2.sh
	./infra/scripts/start-node2.sh

start-node-with-migration: setup-node-db-with-migration
	./infra/scripts/start-node.sh