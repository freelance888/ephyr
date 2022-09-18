<script lang="js">
  import { exportModal } from '../stores';

  import RestreamModal from '../modals/RestreamModal.svelte';
  import PasswordModal from '../modals/PasswordModal.svelte';
  import ExportModal from '../modals/ExportModal.svelte';

  import SettingsModal from '../modals/SettingsModal.svelte';

  import cloneDeep from 'lodash/cloneDeep';
  import { ExportAllRestreams } from '../../api/client.graphql';
  import { showError } from '../utils/util';

  export let info;
  export let state;
  export let isOnline;
  export let gqlClient;

  async function openExportModal() {
    let resp;
    try {
      resp = await gqlClient.query({
        query: ExportAllRestreams,
        fetchPolicy: 'no-cache',
      });
    } catch (e) {
      showError(e.message);
      return;
    }

    if (!!resp.data) {
      exportModal.open(
        null,
        resp.data.export
          ? JSON.stringify(JSON.parse(resp.data.export), null, 2)
          : ''
      );
    }
  }

  let openPasswordModal = false;
  let openSettingsModal = false;
  let openRestreamModal = false;
</script>

<template>
  <a
    href="/"
    class="set-settings"
    on:click|preventDefault={() => (openSettingsModal = true)}
  >
    <i class="fas fa-cog" title="Change settings" />
  </a>
  {#if openSettingsModal}
    <SettingsModal
      info={cloneDeep($info.data.info)}
      bind:visible={openSettingsModal}
    />
  {/if}
  {#key $info.data.info.passwordHash}
    <a
      href="/"
      class="set-password"
      on:click|preventDefault={() => (openPasswordModal = true)}
    >
      <i
        class="fas"
        class:fa-lock-open={!$info.data.info.passwordHash}
        class:fa-lock={!!$info.data.info.passwordHash}
        title="{!$info.data.info.passwordHash ? 'Set' : 'Change'} password"
      />
    </a>
    {#if openPasswordModal}
      <PasswordModal
        password_kind="MAIN"
        current_hash={$info.data.info.passwordHash}
        bind:visible={openPasswordModal}
      />
    {/if}
  {/key}
  <div class="add-input">
    <button
      data-testid="add-input:open-modal-btn"
      class="uk-button uk-button-primary"
      on:click={() => (openRestreamModal = true)}
    >
      <i class="fas fa-plus" />&nbsp;<span>Input</span>
    </button>
    {#if openRestreamModal}
      <RestreamModal
        public_host={$info.data.info.publicHost}
        bind:visible={openRestreamModal}
      />
    {/if}

    {#if isOnline && $state.data}
      <ExportModal />
      <a
        class="export-import-all"
        href="/"
        on:click|preventDefault={openExportModal}
        title="Export/Import all"
      >
        <i class="fas fa-share-square" />
      </a>
    {/if}
  </div>
</template>

<style lang="stylus">
  .set-password, .set-settings
    margin-right: 26px
    font-size: 26px
    color: var(--primary-text-color)
    outline: none

    &:hover
      text-decoration: none
      color: #444

  .add-input
    position: relative
    display: inline-block
    vertical-align: top

  .export-import-all
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
    .export-import-all
      opacity: 1

</style>
