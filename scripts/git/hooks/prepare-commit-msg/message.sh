#!/bin/bash

if [ "$2" == "" ] ; then
	mv $1 $1.tmp
	template_file="$(dirname $0)/message_template.txt"
	issue_no="#$(git branch | grep "*" | awk '{print $2}' | sed -e "s/[^0-9]*/id/\([0-9]*\)/\1/g")"

	message=""
	while read line
	do
		message="${message}#$(eval echo "$line")\n"
	done < $template_file
	message="${message}$(cat $1.tmp)"
	echo -e "$message" > $1.tmp
	cat $1.tmp >> $1
fi
