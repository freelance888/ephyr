<script lang="js">
  import { statusesList } from '../constants/statuses';
  import Confirm from './common/Confirm.svelte';
  import { showError } from '../utils/util';
  import { mutation } from 'svelte-apollo';
  import { RemoveClient } from '../../api/dashboard.graphql';

  const removeClientMutation = mutation(RemoveClient);

  export let client;

  $: clientTitle =
    client.statistics &&
    client.statistics.data &&
    client.statistics.data.clientTitle;

  function getStatusCount(items, status) {
    const filteredItems = items.find((x) => x.status === status);
    return filteredItems ? filteredItems.count : 0;
  }

  async function removeClient() {
    try {
      await removeClientMutation({ variables: { client_id: client.id } });
    } catch (e) {
      showError(e.message);
    }
  }
</script>

<template>
  <section class="uk-section uk-section-muted toolbar">
    <span class="section-label"
      ><a href={client.id}>{clientTitle ? clientTitle : client.id}</a></span
    >
    <Confirm let:confirm>
      <button
        type="button"
        class="uk-close"
        uk-close
        on:click={() => confirm(removeClient)}
      />
      <span slot="title">Removing <code>{client.id}</code> host</span>
      <span slot="description"
        >You won't be able to receive statistics info from this host.</span
      >
      <span slot="confirm">Remove</span>
    </Confirm>
    {#if client.statistics && client.statistics.data}
      <div class="uk-grid uk-grid-small">
        <div class="uk-width-1-2@m uk-width-1-3@s">
          <span class="toolbar-label">
            INPUTS:
            {#each statusesList as status (status)}
              <div
                class="status"
                class:online={status === 'ONLINE'}
                class:offline={status === 'OFFLINE'}
                class:initializing={status === 'INITIALIZING'}
                class:unstable={status === 'UNSTABLE'}
              >
                {getStatusCount(client.statistics.data.inputs, status)}
              </div>
            {/each}
          </span>
        </div>

        <div class="uk-width-expand">
          <span class="toolbar-label">
            OUTPUTS:
            {#each statusesList as status (status)}
              <div
                class="status"
                class:online={status === 'ONLINE'}
                class:offline={status === 'OFFLINE'}
                class:initializing={status === 'INITIALIZING'}
                class:unstable={status === 'UNSTABLE'}
              >
                {getStatusCount(client.statistics.data.outputs, status)}
              </div>
            {/each}
          </span>
        </div>
      </div>
    {:else}
      <div class="uk-alert-danger uk-margin-small">
        {#if !client.statistics}
          <span
            >No statistics. Usually this means that server does not respond.
            Please check the correctness of the server URL</span
          >
        {:else}
          {client.statistics && client.statistics.errors}
        {/if}
      </div>
    {/if}
  </section>
</template>

<style lang="stylus">
  .uk-section
    &:hover
      .uk-close
        opacity: 1
    .uk-alert-danger
      padding: 10px

  .status
    min-width: 28px
    display: inline-flex
    padding-left: 4px

  .uk-close
    position: absolute
    right: -21px
    top: -15px
    opacity: 0
    transition: opacity .3s ease
    &:hover
      opacity: 1
</style>
