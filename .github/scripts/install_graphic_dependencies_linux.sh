#!/bin/bash
set -xeu

VULKAN_SDK_VERSION="1.4.313"

sudo apt-get update -y -qq
wget -qO /usr/share/keyrings/lunarg-archive-keyring.gpg https://packages.lunarg.com/lunarg-signing-key-pub.asc
echo "deb [signed-by=/usr/share/keyrings/lunarg-archive-keyring.gpg] https://packages.lunarg.com/vulkan/$VULKAN_SDK_VERSION noble main" | sudo tee /etc/apt/sources.list.d/lunarg-vulkan-$VULKAN_SDK_VERSION-noble.list
sudo apt-get update -y
sudo apt install -y vulkan-sdk
