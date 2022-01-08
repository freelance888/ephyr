<script lang="js">
  import { createGraphQlClient } from '../utils/util';

  import { Info, State, ServerInfo } from '../../api/client.graphql';
  import { setClient, subscribe } from 'svelte-apollo';
  import Shell from './common/Shell.svelte';
  import Toolbar from './Toolbar.svelte';
  import PageAll from './All.svelte';

  const gqlClient = createGraphQlClient(
    '/api',
    () => (isOnline = true),
    () => (isOnline = false)
  );
  setClient(gqlClient);

  let isOnline = false;
  const info = subscribe(Info, { errorPolicy: 'all' });
  const state = subscribe(State, { errorPolicy: 'all' });
  const serverInfo = subscribe(ServerInfo, { errorPolicy: 'all' });

  $: canRenderToolbar = isOnline && $info.data;
  $: infoError = $info && $info.error;
  $: isLoading = !isOnline || $state.loading;
  $: canRenderMainComponent = isOnline && $state.data && $info.data;
  $: stateError = $state && $state.error;
  $: sInfo = $serverInfo && $serverInfo.data && $serverInfo.data.serverInfo;
  $: document.title = (isOnline ? '' : '🔴  ') + document.title;
</script>

<template>
  <Shell
    {isLoading}
    {canRenderToolbar}
    {canRenderMainComponent}
    error={stateError || infoError}
    serverInfo={sInfo}
  >
    <Toolbar
      slot="toolbar"
      {info}
      {state}
      {serverInfo}
      {isOnline}
      {gqlClient}
    />
    <PageAll slot="main" {info} {state} {serverInfo} />
  </Shell>
</template>
