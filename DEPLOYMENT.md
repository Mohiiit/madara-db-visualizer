# Deployment Plan - Makimono (Madara DB Visualizer)

## Overview

Deploy the full visualizer (frontend + API + sample DB) for public demo access.

```
┌─────────────────────────────────────────────────────────────────┐
│                        ARCHITECTURE                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   GitHub Actions                                                 │
│        │                                                         │
│        ├──► Build Frontend ──► GitHub Pages (Static)            │
│        │                       https://mohiiit.github.io/makimono │
│        │                                                         │
│        └──► Build API Docker ──► Render.com (Free Tier)         │
│                                  https://madara-viz-api.onrender│
│                                         │                        │
│                                         ▼                        │
│                                  ┌─────────────┐                │
│                                  │ Bundled DB  │                │
│                                  │ (888KB)     │                │
│                                  └─────────────┘                │
└─────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Prepare Sample Database (Local)
**Goal:** Create a portable, small database snapshot for demos

### Task 1.1: Copy and verify sample DB
- [ ] Copy current devnet DB to `sample-db/` in repo
- [ ] Verify it contains representative data (blocks, txs, contracts)
- [ ] Add to `.gitignore` if too large, or commit if <5MB

### Task 1.2: Test locally with sample DB
- [ ] Run API with `--db-path ./sample-db`
- [ ] Verify all endpoints work
- [ ] **Feedback:** `curl http://localhost:3000/api/stats` returns valid data

---

## Phase 2: Dockerize the API
**Goal:** Create a Docker image that bundles API + sample DB

### Task 2.1: Create Dockerfile
- [ ] Multi-stage build (builder + runtime)
- [ ] Copy sample-db into image
- [ ] Expose port 3000
- [ ] Set default db-path to bundled DB

### Task 2.2: Test Docker locally
- [ ] `docker build -t madara-viz-api .`
- [ ] `docker run -p 3000:3000 madara-viz-api`
- [ ] **Feedback:** `curl http://localhost:3000/api/health` returns `{"status":"ok"}`

### Task 2.3: Test all features in Docker
- [ ] Schema endpoints work
- [ ] Raw data endpoints work
- [ ] SQL query endpoints work
- [ ] **Feedback:** Run test script that hits all major endpoints

---

## Phase 3: Configure Frontend for Deployment
**Goal:** Make frontend work with remote API

### Task 3.1: Make API URL configurable
- [ ] Read API URL from query param `?api=https://...`
- [ ] Fall back to localStorage
- [ ] Fall back to default (Render URL)
- [ ] Add settings icon to change API URL

### Task 3.2: Build frontend for GitHub Pages
- [ ] Update trunk build with `--public-url /makimono/`
- [ ] Test built files work with remote API
- [ ] **Feedback:** Open `dist/index.html`, check network tab for correct API calls

---

## Phase 4: Set Up GitHub Actions CI/CD
**Goal:** Auto-deploy on push, only if build succeeds

### Task 4.1: Create frontend deploy workflow
- [ ] Trigger on push to main
- [ ] Install Rust + wasm32 target
- [ ] Run `trunk build`
- [ ] **Only deploy if build succeeds**
- [ ] Deploy to GitHub Pages
- [ ] **Feedback:** Check Actions tab, verify green checkmark

### Task 4.2: Create API Docker build workflow
- [ ] Trigger on push to main
- [ ] Build Docker image
- [ ] Push to GitHub Container Registry (ghcr.io)
- [ ] **Only push if build succeeds**
- [ ] **Feedback:** Check Packages tab in GitHub repo

### Task 4.3: Add build status badge to README
- [ ] Add workflow status badges
- [ ] **Feedback:** Badge shows green on README

---

## Phase 5: Deploy API to Render.com
**Goal:** Get API running on free tier

### Task 5.1: Create Render account and service
- [ ] Sign up at render.com (free)
- [ ] Create new "Web Service"
- [ ] Connect to GitHub repo
- [ ] Configure:
  - Build: `docker build`
  - Start: Auto from Dockerfile
  - Free tier (750 hours/month)

### Task 5.2: Configure environment
- [ ] Set PORT=3000 (Render provides PORT env var)
- [ ] Enable auto-deploy on push
- [ ] **Feedback:** Check Render dashboard for deploy logs

### Task 5.3: Verify API is live
- [ ] Get Render URL (e.g., `https://madara-viz-api.onrender.com`)
- [ ] Test: `curl https://madara-viz-api.onrender.com/api/health`
- [ ] Test: `curl https://madara-viz-api.onrender.com/api/stats`
- [ ] **Feedback:** Both return valid JSON

---

## Phase 6: Enable GitHub Pages
**Goal:** Frontend accessible at mohiiit.github.io/makimono

### Task 6.1: Enable GitHub Pages
- [ ] Go to repo Settings → Pages
- [ ] Source: GitHub Actions
- [ ] **Feedback:** Settings shows Pages URL

### Task 6.2: Update frontend default API URL
- [ ] Set default to Render.com URL
- [ ] Commit and push
- [ ] **Feedback:** GitHub Action runs and deploys

### Task 6.3: Verify full deployment
- [ ] Open `https://mohiiit.github.io/makimono`
- [ ] Check blocks load
- [ ] Check schema browser works
- [ ] Check SQL console works
- [ ] **Feedback:** All features work, no console errors

---

## Phase 7: Add Verification & Monitoring
**Goal:** Automated checks to catch issues

### Task 7.1: Add health check to workflow
- [ ] After deploy, curl the API health endpoint
- [ ] Fail workflow if health check fails
- [ ] **Feedback:** Workflow includes health check step

### Task 7.2: Add smoke test script
- [ ] Create `scripts/smoke-test.sh`
- [ ] Test all critical endpoints
- [ ] Run after deployment
- [ ] **Feedback:** Script exits 0 on success

### Task 7.3: Add Uptime monitoring (optional)
- [ ] Use UptimeRobot (free) or similar
- [ ] Monitor API health endpoint
- [ ] Get alerts on downtime
- [ ] **Feedback:** Receive test alert

---

## Execution Order

```
Phase 1 ──► Phase 2 ──► Phase 3 ──► Phase 4 ──► Phase 5 ──► Phase 6 ──► Phase 7
  │           │           │           │           │           │           │
  ▼           ▼           ▼           ▼           ▼           ▼           ▼
Sample DB   Docker     Frontend    CI/CD      Render      GitHub      Verify
 ready      works      config'd    ready      live        Pages       all OK
```

## Quick Commands Reference

```bash
# Phase 1: Test sample DB
./target/release/api --db-path ./sample-db

# Phase 2: Build & run Docker
docker build -t madara-viz-api .
docker run -p 3000:3000 madara-viz-api

# Phase 3: Build frontend
cd crates/frontend && trunk build index.html --release --public-url /makimono/

# Smoke test
curl -s https://YOUR-RENDER-URL/api/health
curl -s https://YOUR-RENDER-URL/api/stats
curl -s https://YOUR-RENDER-URL/api/schema/categories
```

## Rollback Plan

If deployment breaks:
1. Check GitHub Actions logs for build errors
2. Check Render.com logs for runtime errors
3. Revert to last working commit: `git revert HEAD && git push`
4. Render auto-deploys the reverted version

---

## Success Criteria

- [ ] `https://mohiiit.github.io/makimono` loads
- [ ] Can browse blocks, transactions, contracts
- [ ] Schema browser shows all 27 column families
- [ ] SQL console executes queries
- [ ] Raw data browser shows keys
- [ ] No console errors
- [ ] Build badge is green

---

## Notes

- **Render free tier**: Spins down after 15 min inactivity, ~30s cold start
- **DB size**: 888KB, well under limits
- **Estimated time**: 2-3 hours for full deployment
