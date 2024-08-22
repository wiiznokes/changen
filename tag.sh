#!/bin/bash

git tag -d 0.1.6
git push origin :refs/tags/0.1.6
git tag 0.1.6
git push --tags origin