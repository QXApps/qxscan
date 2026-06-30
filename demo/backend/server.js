#!/usr/bin/env node
// server.js — QXScan Demo Backend
// Express HTTPS API with Swagger stubs, PostgreSQL connectivity, TLS 1.3
'use strict';

const https = require('https');
const fs = require('fs');
const path = require('path');
const express = require('express');

const app = express();
app.use(express.json());

// --- Swagger stub ---
const swaggerUi = require('swagger-ui-express');
const swaggerJsdoc = require('swagger-jsdoc');

const swaggerSpec = swaggerJsdoc({
  definition: {
    openapi: '3.0.0',
    info: {
      title: 'QXScan Demo API',
      version: '1.0.0',
      description: 'TLS-secured stub API for QXScan compliance testing',
    },
    servers: [{ url: 'https://backend:443', description: 'Demo backend' }],
    tags: [
      { name: 'Health', description: 'Health check endpoints' },
      { name: 'Users', description: 'User management (stub)' },
      { name: 'Database', description: 'Database operations (stub)' },
      { name: 'Security', description: 'Security posture endpoints' },
    ],
  },
  apis: [__filename],
});

app.use('/api/docs', swaggerUi.serve, swaggerUi.setup(swaggerSpec, {
  customCss: '.swagger-ui .topbar { display: none }',
  customSiteTitle: 'QXScan Demo API — Swagger',
}));

// JSON spec
app.get('/api/openapi.json', (req, res) => res.json(swaggerSpec));

// --- Database connection ---
const { Pool } = require('pg');
const pool = new Pool({
  host: process.env.DB_HOST || 'db',
  port: parseInt(process.env.DB_PORT || '5432'),
  database: process.env.DB_NAME || 'qxscan',
  user: process.env.DB_USER || 'qxscan',
  password: process.env.DB_PASSWORD || 'qxscan_demo_pass',
  ssl: { rejectUnauthorized: false },
  connectionTimeoutMillis: 10000,
});

async function waitForDB(retries = 30, delay = 1000) {
  for (let i = 0; i < retries; i++) {
    try {
      const client = await pool.connect();
      await client.query('SELECT 1');
      client.release();
      console.log(`[server] Database connected (attempt ${i + 1})`);
      return;
    } catch (err) {
      console.log(`[server] Waiting for database (${i + 1}/${retries})...`);
      await new Promise(r => setTimeout(r, delay));
    }
  }
  console.warn('[server] Could not connect to database — running without DB');
}

// --- Swagger-annotated endpoints ---

/**
 * @openapi
 * /api/health:
 *   get:
 *     tags: [Health]
 *     summary: Health check
 *     responses:
 *       200:
 *         description: Service is healthy
 */
app.get('/api/health', (req, res) => {
  res.json({
    status: 'ok',
    service: 'qxscan-demo-backend',
    tls: 'TLS 1.3',
    timestamp: new Date().toISOString(),
  });
});

/**
 * @openapi
 * /api/users:
 *   get:
 *     tags: [Users]
 *     summary: List all users
 *     responses:
 *       200:
 *         description: Array of users
 *       503:
 *         description: Database unavailable
 */
app.get('/api/users', async (req, res) => {
  try {
    const result = await pool.query('SELECT id, name, email, created_at FROM users ORDER BY id');
    res.json(result.rows);
  } catch (err) {
    // Return stub data when DB is not available
    res.json([
      { id: 1, name: 'Alice Johnson (stub)', email: 'alice@demo.quantx.dev', created_at: new Date().toISOString() },
      { id: 2, name: 'Bob Martinez (stub)', email: 'bob@demo.quantx.dev', created_at: new Date().toISOString() },
    ]);
  }
});

/**
 * @openapi
 * /api/stats:
 *   get:
 *     tags: [Database]
 *     summary: Database statistics
 *     responses:
 *       200:
 *         description: Stats object
 */
app.get('/api/stats', async (req, res) => {
  let userCount = 0;
  try {
    const result = await pool.query('SELECT COUNT(*) as count FROM users');
    userCount = parseInt(result.rows[0].count, 10);
  } catch (err) {
    userCount = 2; // stub
  }
  res.json({
    userCount,
    service: 'qxscan-demo-backend',
    tls: 'TLS 1.3',
    database: 'PostgreSQL 16',
    uptime: process.uptime(),
  });
});

/**
 * @openapi
 * /api/security/ciphers:
 *   get:
 *     tags: [Security]
 *     summary: Get supported ciphers (stub)
 *     responses:
 *       200:
 *         description: Supported cipher suites
 */
app.get('/api/security/ciphers', (req, res) => {
  res.json({
    tls_version: 'TLSv1.3',
    ciphers: [
      'TLS_AES_256_GCM_SHA384',
      'TLS_CHACHA20_POLY1305_SHA256',
      'TLS_AES_128_GCM_SHA256',
    ],
    pqc_hybrid: true,
    key_exchange: 'X25519MLKEM768',
    forward_secrecy: true,
  });
});

/**
 * @openapi
 * /api/security/certificate:
 *   get:
 *     tags: [Security]
 *     summary: Get serving certificate info (stub)
 *     responses:
 *       200:
 *         description: Certificate metadata
 */
app.get('/api/security/certificate', (req, res) => {
  res.json({
    subject: { CN: 'backend' },
    issuer: { CN: 'QXScan Demo CA' },
    valid_from: '2026-06-26T00:00:00Z',
    valid_to: '2027-06-26T00:00:00Z',
    key_type: 'ECDSA (prime256v1)',
    signature_algorithm: 'sha256WithECDSAEncryption',
  });
});

// --- Start server ---
const certDir = '/certs';
const options = {
  key: fs.readFileSync(path.join(certDir, 'backend.key')),
  cert: fs.readFileSync(path.join(certDir, 'backend.crt')),
};

async function start() {
  await waitForDB();
  https.createServer(options, app).listen(443, () => {
    console.log('[server] QXScan Demo Backend listening on https://0.0.0.0:443');
    console.log('[server] Swagger docs at https://backend/api/docs');
  });
}

start().catch(err => {
  console.error('[server] Failed to start:', err.message);
  process.exit(1);
});
