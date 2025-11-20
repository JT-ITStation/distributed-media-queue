# 🎬 Distributed Media Queue

Un système de traitement asynchrone de médias (vidéo, audio, image) en Rust avec architecture microservices.

![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Status](https://img.shields.io/badge/status-in--development-yellow.svg)

## 🎯 Objectifs du Projet

Ce projet met en avant :
- **Microservices** : Architecture distribuée avec services découplés
- **API REST asynchrones** : Axum + Tokio pour haute performance
- **Event-Driven Architecture** : Redis Pub/Sub pour communication temps réel
- **Processing asynchrone** : Workers parallèles avec gestion de ressources
- **Monitoring** : WebSocket temps réel + Prometheus metrics
- **Data persistence** : MongoDB avec patterns repository et analytics

## 🏗️ Architecture

```
┌─────────────┐         ┌──────────────┐         ┌──────────────┐
│  API Server │────────▶│    Redis     │────────▶│   Workers    │
│   (Axum)    │         │  (Queue +    │         │ (Video/Audio/│
│             │         │   Pub/Sub)   │         │    Image)    │
└─────────────┘         └──────────────┘         └──────────────┘
       │                       │                          │
       │                       │                          │
       ▼                       ▼                          ▼
┌─────────────┐         ┌──────────────┐         ┌──────────────┐
│   MongoDB   │         │   Monitor    │         │  Prometheus  │
│  (Tasks +   │◀────────│  (WebSocket) │────────▶│   Metrics    │
│   Results)  │         │              │         │              │
└─────────────┘         └──────────────┘         └──────────────┘
```

### Flux de traitement

1. **Client** soumet une tâche via REST API
2. **API Server** valide, sauvegarde dans MongoDB, enqueue dans Redis
3. **Worker** récupère la tâche (BRPOP), traite le média, publie les events
4. **Monitor** broadcast les updates via WebSocket aux clients connectés
5. **Prometheus** collecte les métriques pour analytics
