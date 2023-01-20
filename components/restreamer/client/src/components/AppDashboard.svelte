<script lang="js">
  import Confirm from './common/Confirm.svelte';
  import {createGraphQlClient, showError} from '../utils/util';
  import {mutation, setClient, subscribe} from 'svelte-apollo';
  import Shell from './common/Shell.svelte';
  import { Statistics } from '../../api/dashboard.graphql';
  import ToolbarDashboard from './ToolbarDashboard.svelte';
  import ClientStatistics from './ClientStatistics.svelte';
  import StatusFilter from './common/StatusFilter.svelte';
  import { statusesList } from '../utils/constants';
  import { toggleFilterStatus } from '../utils/filters.util';
  import { EnableAllOutputsForClients, DisableAllOutputsForClients } from '../../api/dashboard.graphql';

  const gqlClient = createGraphQlClient(
    '/api-dashboard',
    () => (isOnline = true),
    () => (isOnline = false)
  );
  setClient(gqlClient);

  let isOnline = false;
  const dashboard = subscribe(Statistics, { errorPolicy: 'all' });
  const enableAllOutputsForClientMutation = mutation(EnableAllOutputsForClients);
  const disableAllOutputsForClientMutation = mutation(DisableAllOutputsForClients);

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

  async function enableAllOutputsOfAllRestreams() {
    try {
      await enableAllOutputsForClientMutation();
    } catch (e) {
      showError(e.message);
    }
  }

  async function disableAllOutputsOfAllRestreams() {
    try {
      await disableAllOutputsForClientMutation();
    } catch (e) {
      showError(e.message);
    }
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
            <span class="toolbar-label">
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
          <div class="uk-flex-auto uk-flex-right uk-flex uk-flex-middle">
            <span class="toolbar-label">
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
          <div class="uk-margin-auto-left">
            <Confirm let:confirm>
              <button
                class="uk-button uk-button-default"
                data-testid="start-all-outputs"
                title="Start all outputs of all restreams"
                on:click={() => confirm(enableAllOutputsOfAllRestreams)}
                ><span>Start All</span>
              </button>
              <span slot="title">Start all outputs</span>
              <span slot="description"
              >This will start all outputs of all restreams.
            </span>
              <span slot="confirm">Start</span>
            </Confirm>

            <Confirm let:confirm>
              <button
                class="uk-button uk-button-default"
                data-testid="stop-all-outputs"
                title="Stop all outputs of all restreams"
                on:click={() => confirm(disableAllOutputsOfAllRestreams)}
                ><span>Stop All</span>
              </button>
              <span slot="title">Stop all outputs</span>
              <span slot="description"
              >This will stop all outputs of all restreams.
              </span>
              <span slot="confirm">Stop</span>
            </Confirm>
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
