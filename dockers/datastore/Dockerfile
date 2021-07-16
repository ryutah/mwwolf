FROM google/cloud-sdk:347.0.0-alpine

RUN apk update && \
	apk --no-cache add openjdk11-jre-headless && \
	gcloud --quiet components install beta cloud-datastore-emulator

CMD ["gcloud","beta","emulators","datastore","start","--host-port","localhost:61000","--project","test"]
