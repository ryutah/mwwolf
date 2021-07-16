#!/bin/bash

set -eu

. $(dirname $0)/../../../../pre-push_conf

check_target_remote(){
	local match_url=false
	for url in ${TARGET_COMMIT_REMOTE_URL}; do
		if [[ "$GIT_HOOKS_URL" == "$url" ]];then
			match_url=true
		fi
	done
	if [[ ! ${match_url} ]]; then
		echo "push対象のremote URLではありません"
	fi
	if [[ "$GIT_HOOKS_REMOTE" != "$TARGET_COMMIT_REMOTE_NAME" ]];then
		echo "push対象のremote名ではありません"
		exit 1
	fi
}

check_prepush(){
	check_target_remote
	exit 0
}



check_prepush
