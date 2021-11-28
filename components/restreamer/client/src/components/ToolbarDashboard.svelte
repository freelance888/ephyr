<script lang="js">
  import AddServerModal from '../modals/AddServerModal.svelte';
  import ExportDashboardModal from '../modals/ExportDashboardModal.svelte';

  export let clients;

  let openAddServerModal = false;
  let openExportModal = false;

  $: hosts = clients.map((x) => x.id);
</script>

<template>
  <div class="add-server">
    <button
      class="uk-button uk-button-primary"
      on:click={() => (openAddServerModal = true)}
    >
      <i class="fas fa-plus" />&nbsp;<span>Add host</span>
    </button>
    {#if openAddServerModal}
      <AddServerModal bind:visible={openAddServerModal} />
    {/if}
    <a
      class="export-import-hosts"
      href="/"
      on:click|preventDefault={() => (openExportModal = true)}
      title="Export/Import hosts"
    >
      <i class="fas fa-share-square" />
    </a>
    {#if openExportModal}
      <ExportDashboardModal {hosts} bind:visible={openExportModal} />
    {/if}
  </div>
</template>

<style lang="stylus">
  .add-server
    position: relative

  .export-import-hosts
    position: absolute
    top: 6px
    right: -24px
    opacity: 0
    transition: opacity .3s ease
    color: var(--primary-text-color)
    outline: none
    &:hover
      text-decoration: none
      color: #444
      opacity: 1

  &:hover
    .export-import-hosts
      opacity: 1
</style>
