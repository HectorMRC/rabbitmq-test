deploy:
	podman-compose -f compose.yaml up -d

follow:
	podman logs --follow --names rabbitmq
	
undeploy:
	podman-compose -f compose.yaml down
