FROM node:22.23.0-bookworm-slim AS build

WORKDIR /app

COPY apps/availability-node/package.json apps/availability-node/package-lock.json ./
RUN npm ci

COPY apps/availability-node/tsconfig.json ./
COPY apps/availability-node/src ./src
RUN npm run build && npm prune --omit=dev

FROM node:22.23.0-bookworm-slim

ENV NODE_ENV=production
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends libatomic1 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/package.json /app/package-lock.json ./
COPY --from=build /app/node_modules ./node_modules
COPY --from=build /app/dist ./dist

RUN mkdir -p \
      /var/lib/dubbridge/p2p-ciphertext \
      /var/lib/dubbridge/p2p-drive \
      /var/lib/dubbridge/p2p-index

EXPOSE 8443

ENTRYPOINT ["node", "dist/main.js"]
