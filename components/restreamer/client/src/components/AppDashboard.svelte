<script lang="js">
  import { createGraphQlClient } from '../utils/util';
  import { setClient, subscribe } from 'svelte-apollo';
  import Shell from './common/Shell.svelte';
  import { Statistics } from '../../api/dashboard.graphql';
  import ToolbarDashboard from './ToolbarDashboard.svelte';
  import ClientStatistics from './ClientStatistics.svelte';
  import StatusFilter from './common/StatusFilter.svelte';
  import { statusesList } from '../utils/constants';
  import { toggleFilterStatus } from '../utils/filters.util';

  const gqlClient = createGraphQlClient(
    '/api-dashboard',
    () => (isOnline = true),
    () => (isOnline = false)
  );
  setClient(gqlClient);

  let isOnline = false;
  const dashboard = subscribe(Statistics, { errorPolicy: 'all' });

  let title = document.title;
  $: document.title = (isOnline ? '' : '🔴  ') + title;

  $: canRenderToolbar = isOnline && $dashboard.data;
  $: error = $dashboard && $dashboard.error;
  $: isLoading = !isOnline || $dashboard.loading;
  $: canRenderMainComponent = isOnline && $dashboard.data;
  $: clients = $dashboard.data && $dashboard.data.statistics;
  $: clientsWithStatistics = clients ? clients.filter((x) => x.statistics) : [];
  $: inputFilters = [];
  $: outputFilters = [];

  $: filteredClients = () => {
    const hasFilteredInputs = (x) =>
      inputFilters.some(
        (status) => getTotalCountByClient(getInputs(x), status) > 0
      );
    const hasFilteredOutputs = (x) =>
      outputFilters.some(
        (status) => getTotalCountByClient(getOutputs(x), status) > 0
      );

    const filtered = inputFilters.length
      ? clientsWithStatistics.filter((client) => hasFilteredInputs(client))
      : clientsWithStatistics;

    return outputFilters.length
      ? filtered.filter((client) => hasFilteredOutputs(client))
      : filtered;
  };

  $: inputStatusCount = (status) =>
    getStatusCount(
      clientsWithStatistics,
      (client) => getInputs(client),
      status
    );

  $: outputStatusCount = (status) =>
    getStatusCount(
      clientsWithStatistics,
      (client) => getOutputs(client),
      status
    );

  const getInputs = (client) => {
    const inputs =
      client.statistics &&
      client.statistics.data &&
      client.statistics.data.inputs;

    return inputs ? inputs : [];
  };

  const getOutputs = (client) => {
    const outputs =
      client.statistics &&
      client.statistics.data &&
      client.statistics.data.outputs;

    return outputs ? outputs : [];
  };

  const getTotalCountByClient = (items, status) => {
    if (!items) {
      return 0;
    }

    const filteredItems = items.find((x) => x.status === status);
    return filteredItems ? filteredItems.count : 0;
  };

  function getStatusCount(allClients, getItems, status) {
    if (!allClients) {
      return 0;
    }

    return allClients
      ? allClients.reduce(
          (sum, client) =>
            (sum += getTotalCountByClient(getItems(client), status)),
          0
        )
      : [];
  }
</script>

<template>
  <Shell {canRenderToolbar} {canRenderMainComponent} {isLoading} {error}>
    <ToolbarDashboard slot="toolbar" {clients} />
    <div slot="main" class="main">
      <section class="uk-section-muted toolbar">
        <span class="section-label">Filters:</span>
        <div class="uk-grid uk-grid-small">
          <div class="uk-width-1-4@m">
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
          <div class="uk-width-1-4@m">
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
