const path = require('path');

/** @type {import('next').NextConfig} */
const distDir = process.env.NEXT_DIST_DIR?.trim() || '.next';

/** @type {import('next').NextConfig} */
const nextConfig = {
  distDir,
  output: 'export',
  trailingSlash: true,
  outputFileTracingRoot: path.join(__dirname),
};

module.exports = nextConfig;
