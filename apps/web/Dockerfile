FROM node:22-alpine AS development
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
EXPOSE 5173
CMD ["npm", "run", "dev"]

FROM development AS build
RUN npm run lint && npm run format:check && npm run build

FROM nginx:stable-alpine AS release
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist /usr/share/nginx/html
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=3s --start-period=10s --retries=5 CMD wget -q -O /dev/null http://127.0.0.1:8080/health/live || exit 1
