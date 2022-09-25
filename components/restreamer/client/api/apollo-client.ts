import { createGraphQlClient } from '../src/utils/util';

export let isOnline = false;
const gqlClient = createGraphQlClient(
  '/api',
  () => (isOnline = true),
  () => (isOnline = false)
);

export default gqlClient;
