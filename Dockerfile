from node:22 as frontend


workdir /usr/local/app/ui
run corepack enable && corepack prepare pnpm@latest --activate
copy ui/package.json ./
run pnpm install
copy ui ./
run npm run build
run cp /usr/local/app/ui/dist /usr/local/app/assets

from rust:1.88 as backend
workdir /usr/local/app
copy . ./
run cargo build --release
