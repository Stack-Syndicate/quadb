cargo license --avoid-dev-deps --json > licenses.json  
python build_licenses.py
rm -rf licenses.json