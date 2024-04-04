import boto3
from boto3.s3.transfer import TransferConfig
from botocore.exceptions import ClientError
import os
import sys
        
PER_UPLOADING = 104857600 ## up to 100 MB per uploading action
GB = 1024 ** 5

def upload_file(file_name, bucket, key):
    object_name = os.path.basename(file_name)

    s3_client = boto3.client('s3')
    try:
        response = s3_client.upload_file(file_name, bucket, key)
    except ClientError as e:
        print(str(e))
        return False
    return True

# https://boto3.amazonaws.com/v1/documentation/api/latest/guide/s3.html
def upload_file_big_file(file_name, bucket, key):
    s3_client = boto3.client('s3')
    config = TransferConfig(multipart_threshold=5*GB, multipart_chunksize=PER_UPLOADING)
    try:
        response = s3_client.upload_file(file_name, bucket, key, Config=config)
    except ClientError as e:
        print(str(e))
        return False
    return True

def upload_file_any_size(file_name, bucket, key):
    file_size = os.path.getsize(file_name)
    if file_size > 5*GB:
        upload_file_big_file(file_name, bucket, key)
    else:
        upload_file(file_name, bucket, key)

def create_bucket(bucket_name, region=None):
    try:
        if region is None:
            s3_client = boto3.client('s3')
            s3_client.create_bucket(Bucket = bucket_name)
        else:
            s3_client = boto3.client('s3', region_name = region)
            location = {'LocationConstraint': region}
            s3_client.create_bucket(Bucket = bucket_name,
                                    CreateBucketConfiguration=location)
    except ClientError as e:
        print(str(e))
        return False
    return True

def list_bucket():
    s3 = boto3.client('s3')
    response = s3.list_buckets()
    print('Existing buckets:')
    for bucket in response['Buckets']:
        print(f'  {bucket["Name"]}')

def list_bucket(bucket_name, max_key):
    s3 = boto3.client('s3')
    response = s3.list_objects(Bucket = bucket_name, MaxKeys = max_key)
    for item in response['Contents']:
        print(f'  {item["Key"]}')


def main():
    bucket_name = sys.argv[1]
    key = sys.argv[2]
    test_file = sys.argv[3]

    print(f"bucket_name: {bucket_name}")
    print(f"key: {key}")
    print(f"test_file: {test_file}")
    upload_file_any_size(test_file, bucket_name, key)
    # list_bucket(bucket_name, 10)

if __name__ == '__main__':
    main()