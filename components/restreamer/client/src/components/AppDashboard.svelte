<script lang="js">
  import { createGraphQlClient } from '../utils/util';
  import { setClient, subscribe } from 'svelte-apollo';
  import Shell from './common/Shell.svelte';
  import { Statistics } from '../../api/dashboard.graphql';
  import ToolbarDashboard from './ToolbarDashboard.svelte';
  import ClientStatistics from './ClientStatistics.svelte';
  import StatusFilter from './common/StatusFilter.svelte';
  import { statusesList } from '../constants/statuses';
  import { toggleFilterStatus } from '../utils/statusFilters.util';

  const gqlClient = createGraphQlClient(
    '/api-dashboard',
    () => (isOnline = true),
    () => (isOnline = false)
  );
  setClient(gqlClient);

  let isOnline = false;

  const dashboard = subscribe(Statistics, { errorPolicy: 'all' });

  $: canRenderToolbar = isOnline && $dashboard.data;
  $: error = $dashboard && $dashboard.error;
  $: isLoading = !isOnline || $dashboard.loading;
  $: canRenderMainComponent = isOnline && $dashboard.data;
  $: clients = $dashboard.data && $dashboard.data.statistics;
  $: inputFilters = [];
  $: outputFilters = [];
  $: filteredClients = () => {
    const hasFilteredInputs = (x) =>
      inputFilters.some(
        (status) => getTotalCountByClient(x.statistics.data.inputs, status) > 0
      );
    const hasFilteredOutputs = (x) =>
      outputFilters.some(
        (status) => getTotalCountByClient(x.statistics.data.outputs, status) > 0
      );

    const filtered = inputFilters.length
      ? clients.filter((client) => hasFilteredInputs(client))
      : clients;

    return outputFilters.length
      ? filtered.filter((client) => hasFilteredOutputs(client))
      : filtered;
  };

  $: inputStatusCount = (status) =>
    getStatusCount(
      clients,
      (client) => (client.statistics ? client.statistics.data.inputs : []),
      status
    );
  $: outputStatusCount = (status) =>
    getStatusCount(
      clients,
      (client) => (client.statistics ? client.statistics.data.outputs : []),
      status
    );

  const getTotalCountByClient = (items, status) => {
    const filteredItems = items.find((x) => x.status === status);
    return filteredItems ? filteredItems.count : 0;
  };

  function getStatusCount(allClients, getItems, status) {
    return allClients
      ? allClients.reduce(
          (sum, client) =>
            (sum += getTotalCountByClient(getItems(client), status)),
          0
        )
      : [];
  }

  // $: console.log(JSON.stringify($dashboard.data))
</script>

<template>
  <Shell {canRenderToolbar} {canRenderMainComponent} {isLoading} {error}>
    <ToolbarDashboard slot="toolbar" {clients} />
    <div slot="main" class="main">
      <section class="uk-section-muted toolbar">
        <span class="section-label">Filters:</span>
        <div class="uk-grid uk-grid-small">
          <div class="uk-width-1-2@m uk-width-1-3@s">
            <span class="toolbar-label total-inputs-label">
              INPUTS:

              {#each statusesList as status (status)}
                <StatusFilter
                  {status}
                  count={inputStatusCount(status)}
                  active={inputFilters.includes(status)}
                  handleClick={() =>
                    (inputFilters = toggleFilterStatus(inputFilters, status))}
                />
              {/each}
            </span>
          </div>
          <div class="uk-width-expand">
            <span class="toolbar-label total-inputs-label">
              OUTPUTS:

              {#each statusesList as status (status)}
                <StatusFilter
                  {status}
                  count={outputStatusCount(status)}
                  active={outputFilters.includes(status)}
                  handleClick={() =>
                    (outputFilters = toggleFilterStatus(outputFilters, status))}
                />
              {/each}
            </span>
          </div>
        </div>
      </section>

      {#each filteredClients() as client}
        <ClientStatistics {client} />
      {/each}
    </div>
  </Shell>
</template>

<style lang="stylus">
  .section-label
    text-transform: uppercase;
    font-weight: bold;
</style>
