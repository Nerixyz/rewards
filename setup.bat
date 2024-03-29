echo Setting up sqlx and database

cargo install sqlx-cli --no-default-features --features postgres
sqlx database setup

echo Compiling web-interface

cd web || exit 1
npm i

REM we don't check the types yet as they have issues (looking at you vuelidate)
npm run vite-build

echo Building setup

cd ..
cargo run -p setup

echo Building server

cargo build -p rewards --release