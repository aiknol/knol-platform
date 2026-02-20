const path = require('path');
const isAdminWebsite = process.env.ADMIN_WEBSITE_MODE === 'true';

/** @type {import('next').NextConfig} */
const nextConfig = {
  // Run website and admin website in parallel without sharing the same .next directory.
  distDir: isAdminWebsite ? '.next-admin' : '.next',
  output: 'export',
  trailingSlash: true,
  outputFileTracingRoot: path.join(__dirname),
  eslint: {
    ignoreDuringBuilds: true,
  },
};

module.exports = nextConfig;
