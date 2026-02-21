function read(name) {
  const value = process.env[name];
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

function fail(message) {
  throw new Error(`[demo env] ${message}`);
}

function assertHttpUrl(name) {
  const value = read(name);
  if (!value) return;
  if (!/^https?:\/\//i.test(value)) {
    fail(`${name} must start with http:// or https://. Received: "${value}"`);
  }
}

function assertHost(name) {
  const value = read(name);
  if (!value) return;
  if (value.includes('/') || value.includes('://')) {
    fail(`${name} must be a host without protocol/path. Received: "${value}"`);
  }
  if (!/^[a-zA-Z0-9.-]+$/.test(value)) {
    fail(`${name} contains invalid host characters. Received: "${value}"`);
  }
}

function assertScheme(name) {
  const value = read(name);
  if (!value) return;
  if (value !== 'http' && value !== 'https') {
    fail(`${name} must be "http" or "https". Received: "${value}"`);
  }
}

const port = read('PORT');
if (port) {
  if (!/^\d+$/.test(port)) {
    fail(`PORT must be numeric. Received: "${port}"`);
  }
  const parsed = Number(port);
  if (!Number.isFinite(parsed) || parsed < 1 || parsed > 65535) {
    fail(`PORT must be between 1 and 65535. Received: "${port}"`);
  }
}

const host = read('HOSTNAME');
if (host && (host.includes('/') || host.includes('://'))) {
  fail(`HOSTNAME must not include protocol/path. Received: "${host}"`);
}

assertScheme('NEXT_PUBLIC_URL_SCHEME');
assertHost('NEXT_PUBLIC_ADMIN_API_HOST');
const adminPort = read('NEXT_PUBLIC_ADMIN_API_PORT');
if (adminPort) {
  if (!/^\d+$/.test(adminPort)) {
    fail(`NEXT_PUBLIC_ADMIN_API_PORT must be numeric. Received: "${adminPort}"`);
  }
  const parsed = Number(adminPort);
  if (!Number.isFinite(parsed) || parsed < 1 || parsed > 65535) {
    fail(`NEXT_PUBLIC_ADMIN_API_PORT must be between 1 and 65535. Received: "${adminPort}"`);
  }
}
assertHttpUrl('NEXT_PUBLIC_ADMIN_API_URL');
