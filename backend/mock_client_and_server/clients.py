#!/usr/bin/env python3
import requests
import json
from concurrent.futures import ThreadPoolExecutor

auth_token='token'
hed = {'Authorization': 'Bearer ' + auth_token, 'Content-Type': 'application/json; charset=UTF-8'}

def post_url(args):
    return requests.post(args[0], data=args[1], headers=hed)
    
payload = {
    "network_type": "GSM",
    "mcc": "1234",
    "mnc": "5678",
    "cid": 12,
    "lac": 12,
    "lat": -1,
    "long": -1,
}
list_of_urls = [("http://localhost:8080/api/positions/cid/1",json.dumps(payload))]*10

with ThreadPoolExecutor(max_workers=10) as pool:
    response_list = list(pool.map(post_url,list_of_urls))

print(json.dumps(payload))

for response in response_list:
    print(response)
    print(response.content)