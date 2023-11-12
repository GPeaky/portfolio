# build front-end
FROM node:21-bookworm-slim AS client
WORKDIR /app/client
COPY . ./
RUN npm install
RUN npm run build

# build back-end
FROM rust:1.73-slim-bookworm AS server
WORKDIR /app/server
RUN apt-get update && apt-get install -y libssl-dev ca-certificates pkg-config && rm -rf /var/lib/apt/lists/*
COPY ./server ./
COPY --from=client /app/client/dist ./dist
RUN cargo build --release
RUN mv ./target/release/portfolio .

CMD ["./portfolio"]