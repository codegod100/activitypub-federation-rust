use crate::{database::DatabaseHandle, generate_object_id, objects::person::DbUser};
use activitypub_federation::{
    activity_queue::send_activity,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::FollowType,
    protocol::context::WithContext,
    traits::{ActivityHandler, Actor},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

use super::accept::Accept;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    pub(crate) actor: ObjectId<DbUser>,
    pub(crate) object: ObjectId<DbUser>,
    #[serde(rename = "type")]
    kind: FollowType,
    id: Url,
}

impl Follow {
    pub fn new(actor: ObjectId<DbUser>, object: ObjectId<DbUser>, id: Url) -> Follow {
        Follow {
            actor,
            object,
            kind: Default::default(),
            id,
        }
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Follow {
    type DataType = DatabaseHandle;
    type Error = crate::error::Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    // Ignore clippy false positive: https://github.com/rust-lang/rust-clippy/issues/6446
    #[allow(clippy::await_holding_lock)]
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        // add to followers
        let local_user = {
            let mut users = data.users.lock().unwrap();
            let local_user = users.first_mut().unwrap();
            local_user.followers.push(self.actor.inner().clone());
            local_user.clone()
        };

        // send back an accept
        let follower = self.actor.dereference(data).await?;
        // let id = generate_object_id(data.domain())?;
        let id = generate_object_id(data.domain())?;
        let accept = Accept::new(local_user.ap_id.clone(), self, id.clone());
        let create_with_context = WithContext::new_default(accept);
        send_activity(
            create_with_context,
            &data.local_user("v"),
            vec![follower.shared_inbox_or_inbox()],
            data,
        )
        .await?;
        // local_user
        //     .send(accept, vec![follower.shared_inbox_or_inbox()], data)
        //     .await?;
        Ok(())
    }
}
