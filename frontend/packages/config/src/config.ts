declare const API_HOST: string;
declare const APP_BUILD_VERSION: string;
declare const BUGSNAG_API_KEY: string;
/* eslint-disable camelcase */
declare const __webpack_public_path__: string;

// For production, API is on the same domain/ip and port as web app, available through sub-route
// i.e. web app is on https://my.healthsupplyhub.com/, then graphql will be available https://my.openmsupply.com/graphql

// For development, API server and front end are launched seperately on different ports and possible different IPs
// by default we assume development API server is launched on the same domain/ip and on port 8007 (Default). We can overwrite this
// with API_HOST which is available through webpack.DefinePlugin (i.e. webpack server --env API_HOST='localhost:9000')

const isProductionBuild = process.env['NODE_ENV'] === 'production';
const { port, hostname, protocol } = window.location;

// Extract the path webpack is serving the app from to use as the base path for react router
// Default to `/` if not defined (e.g. when running in dev mode)
let basePath = '/';
try {
  if (__webpack_public_path__ !== undefined) {
    const url = new URL(__webpack_public_path__);
    basePath = url.pathname;
  }
} catch (e) {
  console.error(e);
}

const defaultDevelopmentApiHost = `${protocol}//${hostname}:8007`;
const productionApiHost = `${protocol}//${hostname}:${port}${basePath}`;

const developmentApiHost =
  (typeof API_HOST !== 'undefined' && API_HOST) || defaultDevelopmentApiHost;

const apiHost = isProductionBuild ? productionApiHost : developmentApiHost;

const version =
  typeof APP_BUILD_VERSION !== 'undefined' && APP_BUILD_VERSION
    ? APP_BUILD_VERSION
    : '0.0.0';
const bugsnagApiKey =
  typeof BUGSNAG_API_KEY !== 'undefined' && BUGSNAG_API_KEY
    ? BUGSNAG_API_KEY
    : '';

export const Environment = {
  API_HOST: apiHost,
  BASE_PATH: basePath,
  BUILD_VERSION: version,
  BUGSNAG_API_KEY: bugsnagApiKey,
};

export default Environment;
