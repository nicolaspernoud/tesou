#!/bin/bash
rm db/*.sqlite
diesel migration run --database-url=db/db.sqlite --migration-dir=db/migrations
