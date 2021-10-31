<script lang="js">
  import { createGraphQlClient } from '../utils/util';
  import { setClient, subscribe } from 'svelte-apollo';
  import Shell from './common/Shell.svelte';
  import { Statistics } from '../../api/dashboard.graphql';
  import ToolbarDashboard from './ToolbarDashboard.svelte';
  import ClientStatistics from './ClientStatistics.svelte';

  const gqlClient = createGraphQlClient(
    '/api-dashboard',
    () => (isOnline = true),
    () => (isOnline = false)
  );
  setClient(gqlClient);

  let isOnline = false;

  const dashboard = subscribe(Statistics, { errorPolicy: 'all' });

  $: canRenderToolbar = isOnline;
  $: error = $dashboard && $dashboard.error;
  $: isLoading = !isOnline || $dashboard.loading;
  $: canRenderMainComponent = isOnline && $dashboard.data;
  $: stat = $dashboard.data && $dashboard.data.statistics;

  // $: console.log(JSON.stringify($dashboard.data))
</script>

<template>
  <Shell {canRenderToolbar} {canRenderMainComponent} {isLoading} {error}>
    <ToolbarDashboard slot="toolbar" />
    <div slot="main" class="main">
      {#each stat as client}
        <ClientStatistics {client} />
      {/each}
    </div>
  </Shell>
</template>

<style lang="stylus">

</style>
