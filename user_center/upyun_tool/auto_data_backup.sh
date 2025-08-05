#!/bin/bash

set -x

# Global vars
install_config_dir='/data'
backup_data_dir='/data'

# Tar backup data
tar czvf $backup_data_dir/backup-$(hostname).tgz $backup_data_dir/backup

# Upload to upyun
/usr/local/bin/upyun_tool --upload $backup_data_dir/backup-$(hostname).tgz $backup_data_dir/backup-$(hostname).tgz --config $install_config_dir/.upyun_pass --server 1
