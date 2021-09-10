<script lang="js">
  import { createGraphQlClient } from '../utils/util';

  import { Info, State } from '../../api/client.graphql';
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

  $: canRenderToolbar = isOnline && $info.data;
  $: infoError = $info && $info.error;
  $: isStateLoading = !isOnline || $state.loading;
  $: canRenderMainComponent = isOnline && $state.data && $info.data;
  $: stateError = $state && $state.error;
</script>

<template>
  <Shell
    {isStateLoading}
    {canRenderToolbar}
    {canRenderMainComponent}
    error={stateError || infoError}
  >
    <Toolbar slot="toolbar" {info} {state} {isOnline} {gqlClient} />
    <PageAll slot="main" {info} {state} />
  </Shell>
</template>
