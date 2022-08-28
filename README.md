# gibd: GIt-ops Backups for grafana Dashboards
This tool exports dashobard definitions from your installation of grafana and stores them in a neat format
gibd creates two directories in your file system: 
* `data` stores dashboard definitions by dashboard uid
* `folders` reflects the state of folders in your grafana. Each file here is a symlink to some file in `data` so renaming your dashboard does not create unnecessary data. This also simlifies the process of restoring a particular dashboard.

# Usage
gibd needs a grafana api token with role `Viewer` supplied as an environment variable and your grafana url supplied as a cli argument
## Example
Create an api token with `Viewer` role, then run ```GRAFANA_TOKEN=mytoken cargo run -- http://localhost:3000```


