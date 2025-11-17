alias cp := commit-push
commit-push MSG:
	git add . && git commit -m "{{MSG}}" && git push
