#!/bin/bash -e

# Copyright (c) The Libra Core Contributors
# SPDX-License-Identifier: Apache-2.0

wget -q https://packages.microsoft.com/config/ubuntu/18.04/packages-microsoft-prod.deb -O /tmp/packages-microsoft-prod.deb
sudo dpkg -i /tmp/packages-microsoft-prod.deb
sudo add-apt-repository universe
sudo apt-get update
sudo apt-get install -y apt-transport-https dotnet-sdk-3.1
rm /tmp/packages-microsoft-prod.deb