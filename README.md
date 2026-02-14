# 🇮🇩 Indonesian Online News API

> Real-time access to **100,000+ Indonesian news articles** from 20+ major sources with NLP-enriched sentiment, emotion, and entity data.

[![RapidAPI](https://img.shields.io/badge/RapidAPI-Available-blue)](https://rapidapi.com)
[![Rust](https://img.shields.io/badge/Rust-Clean_Architecture-orange)](https://www.rust-lang.org)

---

## 🏗️ Architecture

This API is built using **Clean Architecture** principles to ensure maintainability and scalability:

- **Domain Layer**: Core business entities (`NewsArticle`, `SubscriptionTier`) and logic.
- **Infrastructure Layer**: External interfaces (`EsRepository` for Elasticsearch).
- **Service Layer**: Business rules application (`NewsService` for content gating).
- **API Layer**: HTTP handling (`Handlers`, `Routes`, `Middleware`).

---

## 💰 Pricing Tiers (Hourly Limits)

| Tier | Header Value | Price | Limit (Req/Hour) | Content Access | NLP Entities |
|------|--------------|-------|------------------|----------------|--------------|
| **BASIC** | `BASIC` | Free | **5** | Truncated | ❌ |
| **PRO** | `PRO` | $49/mo | **100** | Full Content | ❌ |
| **ULTRA** | `ULTRA` | $99/mo | **1,000** | Full Content | ✅ |
| **MEGA** | `MEGA` | $199/mo | **10,000** | Full Content | ✅ |

> **Note**: Limits are reset every hour at the top of the hour (e.g., 10:00, 11:00).

---

## 🚀 Quick Start

### Subscribe on RapidAPI

1. Go to the [Indonesian Online News API](https://rapidapi.com)
2. Choose a plan: BASIC, PRO, ULTRA, or MEGA
3. Use your `X-RapidAPI-Key`

### Example Request

```bash
curl -X GET "https://indonesian-online-news.p.rapidapi.com/api/news?q=jakarta" \
  -H "X-RapidAPI-Key: YOUR_API_KEY"
```

## 📖 API Reference

### `GET /api/news`
Search news with available filters: `q`, `source`, `tag`, `sentiment`, `emotion`, `author`, `date_from`, `date_to`.

### `GET /api/news/{id}`
Get single article details.

### `GET /api/news/sources`
List all media sources.

### `GET /api/news/stats`
Get dataset statistics.

### `GET /api/news/trending`
Get trending topics (entities & tags).

---

## 🛠️ Self-Hosting

1. **Configure `.env`**:
   ```ini
   RATE_LIMIT_BASIC=5
   RATE_LIMIT_PRO=100
   RATE_LIMIT_ULTRA=1000
   RATE_LIMIT_MEGA=10000
   ```
2. **Run Server**:
   ```bash
   cargo run
   ```

## 📄 License
MIT
