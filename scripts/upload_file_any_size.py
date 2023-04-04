import boto3
from botocore.exceptions import ClientError
import os
import sys
        
per_uploading = 104857600 ## 100 MB per uploading

def upload_file(file_name, bucket, key):
    object_name = os.path.basename(file_name)

    s3_client = boto3.client('s3')
    try:
        response = s3_client.upload_file(file_name, bucket, key)
    except ClientError as e:
        print(str(e))
        return False
    return True

def upload_file_big_file(file_name, bucket):
    client = boto3.client('s3')
    object_name = os.path.basename(file_name)
    try:
        create_bucket(bucket)
        upload_file_big_file(file_name, bucket)
        response = client.create_multipart_upload(Bucket = bucket, Key = object_name)
        upload_id = response["UploadId"]
        file_obj = open(file_name, "rb")
        content = file_obj.read(per_uploading) 
        count = 1
        parts = []
        while content:
            print(count)
            response = client.upload_part(Body = content, Bucket = bucket, Key = object_name, PartNumber = count, UploadId = upload_id)
            parts.append({'ETag': response["ETag"], 'PartNumber': count})
            count += 1
            content = file_obj.read(per_uploading) ## 100 MB per uploading
        response = client.complete_multipart_upload(Bucket = bucket, Key = object_name, MultipartUpload = {'Parts': parts,}, UploadId = upload_id)
        print(str(response))
    except ClientError as e:
        print(str(e))
        return False
    return True

def upload_file_any_size(file_name, bucket, key):
    file_size = os.path.getsize(file_name)
    if file_size > per_uploading:
        upload_file_big_file(file_name, bucket, key)
    else:
        upload_file(file_name, bucket, key)

def create_bucket(bucket_name, region=None):
    try:
        if region is None:
            s3_client = boto3.client('s3')
            s3_client.create_bucket(Bucket=bucket_name)
        else:
            s3_client = boto3.client('s3', region_name=region)
            location = {'LocationConstraint': region}
            s3_client.create_bucket(Bucket=bucket_name,
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

    upload_file_any_size(test_file, bucket_name, key)
    list_bucket(bucket_name, 10)

if __name__ == '__main__':
    main()