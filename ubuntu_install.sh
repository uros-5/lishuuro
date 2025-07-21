

# nvm install

curl -sL https://raw.githubusercontent.com/nvm-sh/nvm/v0.35.0/install.sh -o install_nvm.sh
bash install_nvm.sh 

nvm
source ~/.bashrc 

nvm --versionn
nvm install --lts

curl -fsSL https://get.pnpm.io/install.sh | sh -
source ~/.bashrc 

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc
rustc --version

curl -fsSL https://www.mongodb.org/static/pgp/server-8.0.asc |    sudo gpg -o /usr/share/keyrings/mongodb-server-8.0.gpg    --dearmor
curl -fsSL https://www.mongodb.org/static/pgp/server-8.0.asc |    sudo gpg -o /usr/share/keyrings/mongodb-server-8.0.gpg    --dearmor
cat /etc/lsb-release
echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-8.0.gpg ] https://repo.mongodb.org/apt/ubuntu noble/mongodb-org/8.0 multiverse" | sudo tee /etc/apt/sources.list.d/mongodb-org-8.0.list
sudo apt-get update
sudo apt-get install -y mongodb-org
sudo systemctl start mongod
sudo apt-get install lsb-release curl gpg


curl -fsSL https://packages.redis.io/gpg | sudo gpg --dearmor -o /usr/share/keyrings/redis-archive-keyring.gpg
sudo chmod 644 /usr/share/keyrings/redis-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/redis-archive-keyring.gpg] https://packages.redis.io/deb $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/redis.list
sudo apt-get update
sudo apt-get install redis
[200~sudo systemctl enable redis-server
sudo systemctl enable redis-server
sudo systemctl start redis-server

sudo apt install build-essential
sudo apt install libssl-dev
sudo apt install pkg-config
sudo apt-get install libudev-dev


# optional
#
## not-tw
#
# wget https://github.com/uros-5/not-tailwind/releases/download/v0.2.0/not-tailwind_v0.2.0_x86_64-unknown-linux-musl.tar.gz
# tar -xzf not-tailwind_v0.2.0_x86_64-unknown-linux-musl.tar.gz 
# cd not-tailwind
# mv not-tailwind /usr/bin/
# cd ..
# rm -rf not-tailwind_v0.2.0_x86_64-unknown-linux-musl.tar.gz 
#
## fd
#  
# wget https://github.com/sharkdp/fd/releases/download/v10.2.0/fd-v10.2.0-x86_64-unknown-linux-gnu.tar.gz
# tar -xzf fd-v10.2.0-x86_64-unknown-linux-gnu.tar.gz 
# ls
# cd fd-v10.2.0-x86_64-unknown-linux-gnu/
# ls
# mv fd /usr/bin
# cd ..
 
# 
## certbot installation
# apt install snapd
# sudo snap install --classic certbot
# sudo ln -s /snap/bin/certbot /usr/bin/certbot
# sudo certbot certonly --standalone
#
# apt install certbot python-certbot-nginx
# apt install certbot python3-certbot-nginx
# certbot --nginx -d lishuuro.org
# 
#
## ngnix installation
# sudo apt install nginx
# sudo ufw app list
# sudo ufw allow 'Nginx HTTP'
# sudo ufw status
# systemctl status nginx

