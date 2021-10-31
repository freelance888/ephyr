<script lang="js">
  import AddServerModal from '../modals/AddServerModal.svelte';
  let openAddServerModal = false;
</script>

<template>
  <div class="add-server">
    <button
      class="uk-button uk-button-primary"
      on:click={() => (openAddServerModal = true)}
    >
      <i class="fas fa-plus" />&nbsp;<span>Add server</span>
    </button>
    {#if openAddServerModal}
      <AddServerModal bind:visible={openAddServerModal} />
    {/if}
  </div>
</template>

<style lang="stylus">

</style>
