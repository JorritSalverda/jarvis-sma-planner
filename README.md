## Installation

To install this application using Helm run the following commands:

```bash
helm repo add jorritsalverda https://helm.jorritsalverda.com
kubectl create namespace jarvis-sma-planner

helm upgrade \
  jarvis-sma-planner \
  jorritsalverda/jarvis-sma-planner \
  --install \
  --namespace jarvis-sma-planner \
  --wait
```
